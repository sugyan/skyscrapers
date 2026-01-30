use crate::LatinSquare;
use crate::jacobson_matthews::JMState;
use rand::Rng;

/// Parameters for the MCMC sampler.
#[derive(Debug, Clone)]
pub struct SamplerParams {
    /// Burn-in steps to reach equilibrium from initial cyclic state.
    ///
    /// If `None`, uses n³ as the burn-in value. The mixing time of
    /// Jacobson-Matthews is empirically observed to be O(n³ log n),
    /// though not rigorously proven. Using n³ provides good uniformity
    /// in practice.
    pub burn_in: Option<u64>,
    /// Number of steps between successive samples (for iterator mode, v0.2+).
    /// Not used by one-shot `sample()`.
    pub steps: u64,
    /// Thinning factor (for iterator mode, v0.2+).
    pub thinning: u64,
    /// Probability of doing nothing (for aperiodicity).
    pub p_do_nothing: f64,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            burn_in: None, // auto: n³
            steps: 1_000,
            thinning: 1,
            p_do_nothing: 0.01,
        }
    }
}

/// Generates an approximately uniform Latin square of order `n`.
///
/// Uses MCMC sampling with the Jacobson-Matthews algorithm for ergodicity.
/// The output is deterministic given the same seed and parameters.
///
/// # Panics
/// Panics if:
/// - `n < 2` or `n > 255`
/// - `p_do_nothing` is not in `[0.0, 1.0]`
pub fn sample<R: Rng + ?Sized>(n: usize, rng: &mut R, params: &SamplerParams) -> LatinSquare {
    assert!((2..=255).contains(&n), "n must be in range 2..=255");
    assert!(
        (0.0..=1.0).contains(&params.p_do_nothing),
        "p_do_nothing must be in [0.0, 1.0]"
    );

    let mut state = JMState::new_cyclic(n);

    // burn_in: steps to reach equilibrium from initial cyclic state
    // Default to n³ if not specified
    let burn_in = params.burn_in.unwrap_or((n * n * n) as u64);
    for _ in 0..burn_in {
        step(&mut state, rng, params);
    }

    // Ensure we return a proper Latin square
    while !state.is_proper() {
        state.step(rng);
    }

    state.to_latin_square()
}

/// Performs a single MCMC step using Jacobson-Matthews.
fn step<R: Rng + ?Sized>(state: &mut JMState, rng: &mut R, params: &SamplerParams) {
    // With probability p_do_nothing: do nothing (for aperiodicity)
    if rng.random::<f64>() < params.p_do_nothing {
        return;
    }

    state.step(rng);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn quick_params() -> SamplerParams {
        SamplerParams {
            burn_in: Some(1000),
            steps: 500,
            thinning: 1,
            p_do_nothing: 0.01,
        }
    }

    #[test]
    fn reproducibility_same_seed_same_output() {
        let seed = [0u8; 32];
        let params = quick_params();

        let mut rng1 = ChaCha20Rng::from_seed(seed);
        let sq1 = sample(7, &mut rng1, &params);

        let mut rng2 = ChaCha20Rng::from_seed(seed);
        let sq2 = sample(7, &mut rng2, &params);

        assert_eq!(sq1, sq2, "Same seed should produce identical squares");
    }

    #[test]
    fn different_seed_different_output_smoke() {
        let params = quick_params();

        // Try a few different seed pairs
        for offset in 0u8..5 {
            let mut seed1 = [0u8; 32];
            seed1[0] = offset;
            let mut seed2 = [0u8; 32];
            seed2[0] = offset + 100;

            let mut rng1 = ChaCha20Rng::from_seed(seed1);
            let sq1 = sample(7, &mut rng1, &params);

            let mut rng2 = ChaCha20Rng::from_seed(seed2);
            let sq2 = sample(7, &mut rng2, &params);

            if sq1 != sq2 {
                return; // Success: found different outputs
            }
        }
        panic!("All tested seed pairs produced identical squares (extremely unlikely)");
    }
}
