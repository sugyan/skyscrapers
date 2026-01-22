use rand::Rng;

use crate::LatinSquare;

/// Parameters for the MCMC sampler.
#[derive(Debug, Clone)]
pub struct SamplerParams {
    /// Number of steps discarded (burn-in phase).
    pub burn_in: u64,
    /// Number of steps after burn-in before returning.
    pub steps: u64,
    /// Thinning factor (for iterator mode, v0.2+).
    pub thinning: u64,
    /// Probability of choosing a row move (vs column move).
    pub p_row_move: f64,
    /// Probability of doing nothing (for aperiodicity).
    pub p_do_nothing: f64,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            burn_in: 300_000,
            steps: 80_000,
            thinning: 1,
            p_row_move: 0.5,
            p_do_nothing: 0.01,
        }
    }
}

/// Generates an approximately uniform Latin square of order `n`.
///
/// Uses MCMC sampling with the given RNG and parameters.
/// The output is deterministic given the same seed and parameters.
///
/// # Panics
/// Panics if `n < 2` or `n > 255`.
pub fn sample<R: Rng + ?Sized>(n: usize, rng: &mut R, params: &SamplerParams) -> LatinSquare {
    todo!()
}
