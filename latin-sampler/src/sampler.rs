use crate::LatinSquare;
use crate::jacobson_matthews::JMState;
use rand::Rng;

/// Parameters for the MCMC sampler.
#[derive(Debug, Clone)]
pub struct SamplerParams {
    /// Burn-in steps to reach equilibrium from initial cyclic state.
    ///
    /// This is the number of MCMC steps (including do-nothing steps with
    /// probability `p_do_nothing`). The expected number of actual Jacobson-Matthews
    /// moves is `burn_in * (1 - p_do_nothing)`.
    ///
    /// If `None`, uses n³ as the burn-in value. The mixing time of
    /// Jacobson-Matthews is empirically observed to be O(n³ log n),
    /// though not rigorously proven. Using n³ provides good uniformity
    /// in practice.
    pub burn_in: Option<u64>,
    /// Number of steps between successive samples (for iterator mode, v0.2+).
    /// Not used by one-shot `sample()`.
    pub steps: u64,
    /// Thinning factor: steps between successive samples.
    ///
    /// If `None`, uses 3×n² for approximate independence. Based on ACF
    /// analysis, this achieves IACT τ ≤ 1.10 and |ACF(1)| ≤ 0.05 for all n.
    pub thinning: Option<u64>,
    /// Probability of doing nothing (for aperiodicity).
    pub p_do_nothing: f64,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            burn_in: None, // auto: n³
            steps: 1_000,
            thinning: None, // auto: 3×n²
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
        step(&mut state, rng, params);
    }

    state.to_latin_square()
}

/// An iterator that produces approximately uniform Latin squares.
///
/// Created by [`Sampler::new`]. Uses MCMC sampling with the Jacobson-Matthews
/// algorithm for ergodicity. Burn-in is performed on the first call to `next()`.
///
/// # Example
///
/// ```
/// use latin_sampler::{Sampler, SamplerParams};
/// use rand_chacha::ChaCha20Rng;
/// use rand::SeedableRng;
///
/// let rng = ChaCha20Rng::from_seed([0u8; 32]);
/// let params = SamplerParams::default();
/// let sampler = Sampler::new(7, rng, params);
///
/// for sq in sampler.take(10) {
///     println!("Cell (0,0) = {}", sq.get(0, 0));
/// }
/// ```
pub struct Sampler<R> {
    n: usize,
    state: JMState,
    rng: R,
    params: SamplerParams,
    burned_in: bool,
}

impl<R: Rng> Sampler<R> {
    /// Create a new sampler for Latin squares of order `n`.
    ///
    /// The sampler starts from a cyclic Latin square and performs burn-in
    /// on the first call to `next()`.
    ///
    /// # Panics
    /// Panics if:
    /// - `n < 2` or `n > 255`
    /// - `p_do_nothing` is not in `[0.0, 1.0]`
    pub fn new(n: usize, rng: R, params: SamplerParams) -> Self {
        assert!((2..=255).contains(&n), "n must be in range 2..=255");
        assert!(
            (0.0..=1.0).contains(&params.p_do_nothing),
            "p_do_nothing must be in [0.0, 1.0]"
        );

        Self {
            n,
            state: JMState::new_cyclic(n),
            rng,
            params,
            burned_in: false,
        }
    }
}

impl<R: Rng> Iterator for Sampler<R> {
    type Item = LatinSquare;

    fn next(&mut self) -> Option<Self::Item> {
        // First call: perform burn-in
        if !self.burned_in {
            let burn_in = self
                .params
                .burn_in
                .unwrap_or((self.n * self.n * self.n) as u64);
            for _ in 0..burn_in {
                step(&mut self.state, &mut self.rng, &self.params);
            }
            self.burned_in = true;
        } else {
            // Subsequent calls: apply thinning steps
            // Default thinning is 3×n² for approximate independence
            let thinning = self.params.thinning.unwrap_or((3 * self.n * self.n) as u64);
            for _ in 0..thinning {
                step(&mut self.state, &mut self.rng, &self.params);
            }
        }

        // Ensure we return a proper Latin square
        while !self.state.is_proper() {
            step(&mut self.state, &mut self.rng, &self.params);
        }

        Some(self.state.to_latin_square())
    }
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
            thinning: Some(1),
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

    #[test]
    fn iterator_reproducibility() {
        let seed = [0u8; 32];
        let params = quick_params();

        // Create two samplers with the same seed
        let rng1 = ChaCha20Rng::from_seed(seed);
        let sampler1 = Sampler::new(5, rng1, params.clone());

        let rng2 = ChaCha20Rng::from_seed(seed);
        let sampler2 = Sampler::new(5, rng2, params);

        // Take 10 samples from each
        let squares1: Vec<_> = sampler1.take(10).collect();
        let squares2: Vec<_> = sampler2.take(10).collect();

        assert_eq!(
            squares1, squares2,
            "Same seed should produce identical sequence"
        );
    }

    #[test]
    fn iterator_thinning_spacing() {
        let seed = [0u8; 32];

        // Create params with different thinning values
        let params_thin1 = SamplerParams {
            burn_in: Some(1000),
            thinning: Some(1),
            ..Default::default()
        };
        let params_thin100 = SamplerParams {
            burn_in: Some(1000),
            thinning: Some(100),
            ..Default::default()
        };

        let rng1 = ChaCha20Rng::from_seed(seed);
        let sampler1 = Sampler::new(5, rng1, params_thin1);

        let rng2 = ChaCha20Rng::from_seed(seed);
        let sampler2 = Sampler::new(5, rng2, params_thin100);

        // Take 5 samples from each
        let squares1: Vec<_> = sampler1.take(5).collect();
        let squares2: Vec<_> = sampler2.take(5).collect();

        // First sample should be the same (same burn-in)
        assert_eq!(
            squares1[0], squares2[0],
            "First sample after burn-in should be identical"
        );

        // Subsequent samples should differ due to different thinning
        let different_count = squares1
            .iter()
            .zip(squares2.iter())
            .skip(1)
            .filter(|(a, b)| a != b)
            .count();

        assert!(
            different_count > 0,
            "Different thinning should produce different sequences after first sample"
        );
    }
}
