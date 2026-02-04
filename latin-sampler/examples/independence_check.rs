//! Independence Check Example
//!
//! Verifies that the default thinning (3×n²) achieves approximate independence
//! by computing IACT (τ) and ACF(1) for various n values.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --release --example independence_check
//! ```

use latin_sampler::{Sampler, SamplerParams};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// Observation Functions
// ============================================================================

/// Generate deterministic random coefficients for linear projection
fn generate_linear_coefficients(n: usize) -> Vec<f64> {
    // Use a simple LCG for deterministic coefficient generation
    let mut state: u64 = 12345;
    let a: u64 = 6364136223846793005;
    let c: u64 = 1442695040888963407;

    let num_cells = n * n;
    let mut coeffs = Vec::with_capacity(num_cells);

    for _ in 0..num_cells {
        state = state.wrapping_mul(a).wrapping_add(c);
        // Map to [-1, 1]
        let val = (state as f64 / u64::MAX as f64) * 2.0 - 1.0;
        coeffs.push(val);
    }

    coeffs
}

/// Linear projection g(X) = Σ w_k * cell_k
fn g_linear(cells: &[u8], coeffs: &[f64]) -> f64 {
    cells
        .iter()
        .zip(coeffs.iter())
        .map(|(&c, &w)| c as f64 * w)
        .sum()
}

/// Hash-based observation function (maps state to [0, 1))
fn g_hash(cells: &[u8]) -> f64 {
    let mut hasher = DefaultHasher::new();
    cells.hash(&mut hasher);
    let h = hasher.finish();
    h as f64 / u64::MAX as f64
}

// ============================================================================
// ACF and IACT Computation
// ============================================================================

/// Compute autocorrelation function for lags 1..max_lag
fn compute_acf(series: &[f64], max_lag: usize) -> Vec<f64> {
    let n = series.len();
    if n < 2 {
        return vec![0.0; max_lag];
    }

    // Compute mean
    let mean: f64 = series.iter().sum::<f64>() / n as f64;

    // Compute variance
    let var: f64 = series.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;

    if var < 1e-15 {
        return vec![0.0; max_lag];
    }

    // Compute ACF for each lag
    let mut acf = Vec::with_capacity(max_lag);
    for lag in 1..=max_lag {
        if lag >= n {
            acf.push(0.0);
            continue;
        }
        let cov: f64 = series
            .iter()
            .take(n - lag)
            .zip(series.iter().skip(lag))
            .map(|(&x, &y)| (x - mean) * (y - mean))
            .sum::<f64>()
            / (n - lag) as f64;
        acf.push(cov / var);
    }

    acf
}

/// Sokal's automatic windowing method for IACT estimation
fn compute_iact_sokal(acf: &[f64], c: f64) -> f64 {
    if acf.is_empty() {
        return 1.0;
    }

    let mut tau = 1.0;
    for (m, &rho) in acf.iter().enumerate() {
        let m1 = m + 1;
        tau += 2.0 * rho;

        let tau_eff = tau.max(1.0);
        if (m1 as f64) >= c * tau_eff {
            return tau_eff;
        }
    }

    tau.max(1.0)
}

// ============================================================================
// Independence Test
// ============================================================================

struct TestResult {
    n: usize,
    thinning: u64,
    samples: usize,
    tau: f64,
    abs_rho1: f64,
}

fn run_test(n: usize, samples: usize, seed: u64) -> TestResult {
    let params = SamplerParams::default();
    let thinning = params.thinning.unwrap_or((3 * n * n) as u64);

    let rng = ChaCha20Rng::seed_from_u64(seed);
    let sampler = Sampler::new(n, rng, params);

    let linear_coeffs = generate_linear_coefficients(n);

    let mut linear_series = Vec::with_capacity(samples);
    let mut hash_series = Vec::with_capacity(samples);

    for sq in sampler.take(samples) {
        let cells = sq.cells();
        linear_series.push(g_linear(cells, &linear_coeffs));
        hash_series.push(g_hash(cells));
    }

    let max_lag = 100;
    let c = 5.0;

    // Compute ACF and IACT for both observation functions
    let linear_acf = compute_acf(&linear_series, max_lag);
    let hash_acf = compute_acf(&hash_series, max_lag);

    let linear_tau = compute_iact_sokal(&linear_acf, c);
    let hash_tau = compute_iact_sokal(&hash_acf, c);

    let linear_rho1 = linear_acf.first().copied().unwrap_or(0.0).abs();
    let hash_rho1 = hash_acf.first().copied().unwrap_or(0.0).abs();

    // Return worst case
    let tau = linear_tau.max(hash_tau);
    let abs_rho1 = linear_rho1.max(hash_rho1);

    TestResult {
        n,
        thinning,
        samples,
        tau,
        abs_rho1,
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("Independence Check (thinning = 3*n^2)");
    println!("=====================================");
    println!();

    let test_cases = [
        (5, 10000),
        (7, 10000),
        (9, 10000),
        (11, 10000),
        (15, 5000),
        (20, 3000),
        (26, 2000),
    ];

    let seed = 42u64;
    let mut all_passed = true;

    for (n, samples) in test_cases {
        let result = run_test(n, samples, seed);

        let tau_ok = result.tau <= 1.10;
        let rho1_ok = result.abs_rho1 <= 0.05;
        let passed = tau_ok && rho1_ok;

        if !passed {
            all_passed = false;
        }

        println!(
            "n={:<2} (thin={:<4}): samples={:<5} tau={:.2}  |rho1|={:.3}{}",
            result.n,
            result.thinning,
            result.samples,
            result.tau,
            result.abs_rho1,
            if passed { "" } else { "  [FAIL]" }
        );
    }

    println!();
    if all_passed {
        println!("All tau <= 1.10 and |rho1| <= 0.05: Approximate independence achieved.");
    } else {
        println!("WARNING: Some tests did not meet independence criteria.");
    }
}
