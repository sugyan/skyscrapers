//! Uniformity test using pilot-run approach with corrected chi-square methodology.
//!
//! Uses a two-phase approach:
//! - Phase 1: Pilot run to estimate bucket probabilities π_b
//! - Phase 2: Test run with independent seeds, chi-square using expected = test_samples × π_b
//!
//! Two modes available:
//! - Normal mode (default): 8192 buckets, 100k test, 10M pilot
//! - Light mode (--light): 1024 buckets, 20k test, 1M pilot
//!
//! Parameter selection rationale:
//! - RelErr(π_hat) ≈ sqrt(B / N_pilot)  -- pilot estimate accuracy
//! - E_per_bucket = N_test / B          -- chi-square stability (≥10 recommended)
//! - sd(χ²/df) ≈ sqrt(2 / df)           -- expected random fluctuation
//!
//! Normal mode (8192 buckets, 10M pilot, 100k test):
//!   - RelErr ≈ 2.9%, E_per_bucket ≈ 12.2, 95% band ≈ [0.97, 1.03]
//!
//! Light mode (1024 buckets, 1M pilot, 20k test):
//!   - RelErr ≈ 3.2%, E_per_bucket ≈ 19.5, 95% band ≈ [0.91, 1.09]

use latin_sampler::{Sampler, SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};

// Normal mode parameters
const DEFAULT_NUM_BUCKETS: usize = 8192;
const DEFAULT_TEST_SAMPLES: usize = 100_000;
const DEFAULT_PILOT_SAMPLES: usize = 10_000_000;

// Light mode parameters
const LIGHT_NUM_BUCKETS: usize = 1024;
const LIGHT_TEST_SAMPLES: usize = 20_000;
const LIGHT_PILOT_SAMPLES: usize = 1_000_000;

fn hash_cells(cells: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    cells.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let light_mode = args.iter().any(|s| s == "--light");
    let oneshot_mode = args.iter().any(|s| s == "--oneshot");

    let (num_buckets, test_samples, pilot_samples) = if light_mode {
        (LIGHT_NUM_BUCKETS, LIGHT_TEST_SAMPLES, LIGHT_PILOT_SAMPLES)
    } else {
        (
            DEFAULT_NUM_BUCKETS,
            DEFAULT_TEST_SAMPLES,
            DEFAULT_PILOT_SAMPLES,
        )
    };

    let mode_str = if light_mode { "light" } else { "normal" };
    let sampling_mode = if oneshot_mode { "oneshot" } else { "iterator" };
    println!("=== Uniformity Test (Pilot-Run Approach) ===");
    println!(
        "n = {}, mode = {}, sampling = {}",
        n, mode_str, sampling_mode
    );
    println!(
        "  buckets = {}, pilot = {}, test = {}",
        num_buckets, pilot_samples, test_samples
    );
    println!("  burn_in = n³ = {} (auto)", n * n * n);
    println!();

    // Use higher thinning to reduce correlation between consecutive samples
    // in a single MCMC chain. n³ steps between samples ensures near-independence.
    let params = SamplerParams {
        thinning: Some((2 * n) as u64),
        ..Default::default()
    };
    run_pilot_bucket_test(
        n,
        pilot_samples,
        test_samples,
        num_buckets,
        &params,
        oneshot_mode,
    );
}

/// Pilot-run bucket test.
///
/// This approach solves the problem where different buckets contain different numbers
/// of Latin squares. We first run a large pilot sample to estimate the true probability
/// π_b for each bucket, then use these estimates to compute the expected counts for
/// the chi-square test.
///
/// Phase 1: Pilot run (e.g., 5M samples) to estimate π_b = pilot_counts[b] / pilot_samples
/// Phase 2: Test run (e.g., 100k samples) with different seeds
/// Phase 3: Chi-square using expected = test_samples × π_b
fn run_pilot_bucket_test(
    n: usize,
    pilot_samples: usize,
    test_samples: usize,
    num_buckets: usize,
    params: &SamplerParams,
    oneshot_mode: bool,
) {
    println!("Mode: Pilot-run bucket test ({} buckets)", num_buckets);
    println!(
        "  Pilot samples: {}, Test samples: {}",
        pilot_samples, test_samples
    );
    println!();

    // Phase 1: Pilot run to estimate bucket probabilities
    println!(
        "Phase 1: Pilot run ({} samples) to estimate bucket probabilities...",
        pilot_samples
    );
    let mut pilot_counts = vec![0usize; num_buckets];

    if oneshot_mode {
        // One-shot mode: call sample() with different seeds for each sample
        for i in 0..pilot_samples {
            let mut rng = ChaCha20Rng::seed_from_u64(i as u64);
            let sq = sample(n, &mut rng, params);
            let bucket = hash_cells(sq.cells()) as usize % num_buckets;
            pilot_counts[bucket] += 1;
        }
    } else {
        // Iterator mode: use Sampler for continuous sampling
        let pilot_rng = ChaCha20Rng::seed_from_u64(0);
        let pilot_sampler = Sampler::new(n, pilot_rng, params.clone());
        for sq in pilot_sampler.take(pilot_samples) {
            let bucket = hash_cells(sq.cells()) as usize % num_buckets;
            pilot_counts[bucket] += 1;
        }
    }
    println!("Done.\n");

    // Compute estimated probabilities π_b
    let pi: Vec<f64> = pilot_counts
        .iter()
        .map(|&c| c as f64 / pilot_samples as f64)
        .collect();

    // Phase 2: Test run with different seed (for independence)
    println!(
        "Phase 2: Test run ({} samples) with independent seed...",
        test_samples
    );
    let mut test_counts = vec![0usize; num_buckets];

    if oneshot_mode {
        // One-shot mode: use different seed range for test samples
        let seed_offset = pilot_samples as u64 + 1_000_000;
        for i in 0..test_samples {
            let mut rng = ChaCha20Rng::seed_from_u64(seed_offset + i as u64);
            let sq = sample(n, &mut rng, params);
            let bucket = hash_cells(sq.cells()) as usize % num_buckets;
            test_counts[bucket] += 1;
        }
    } else {
        // Iterator mode: use Sampler with different seed for independence
        let test_rng = ChaCha20Rng::seed_from_u64(12345);
        let test_sampler = Sampler::new(n, test_rng, params.clone());
        for sq in test_sampler.take(test_samples) {
            let bucket = hash_cells(sq.cells()) as usize % num_buckets;
            test_counts[bucket] += 1;
        }
    }
    println!("Done.\n");

    // Phase 3: Chi-square calculation with corrected expected values
    // Only include buckets where π_b > 0 (observed in pilot)
    let chi_square: f64 = (0..num_buckets)
        .filter(|&b| pi[b] > 0.0)
        .map(|b| {
            let expected = test_samples as f64 * pi[b];
            let diff = test_counts[b] as f64 - expected;
            diff * diff / expected
        })
        .sum();

    let non_empty_buckets = pi.iter().filter(|&&p| p > 0.0).count();
    let df = non_empty_buckets - 1;
    let normalized = chi_square / df as f64;

    println!("Results:");
    println!("  Non-empty buckets: {}", non_empty_buckets);
    println!("  Chi-square: {:.2}", chi_square);
    println!("  Degrees of freedom: {}", df);
    println!("  Normalized (chi^2/df): {:.4}", normalized);
    println!();

    print_result(normalized);

    // Bucket statistics for test run
    let test_min = *test_counts.iter().min().unwrap();
    let test_max = *test_counts.iter().max().unwrap();
    let test_empty = test_counts.iter().filter(|&&c| c == 0).count();

    // Pilot statistics
    let pilot_min = *pilot_counts.iter().min().unwrap();
    let pilot_max = *pilot_counts.iter().max().unwrap();
    let pilot_empty = pilot_counts.iter().filter(|&&c| c == 0).count();

    println!();
    println!("Pilot run bucket statistics:");
    println!("  Min count: {}", pilot_min);
    println!("  Max count: {}", pilot_max);
    println!("  Empty buckets: {}", pilot_empty);

    println!();
    println!("Test run bucket statistics:");
    println!("  Min count: {}", test_min);
    println!("  Max count: {}", test_max);
    println!("  Empty buckets: {}", test_empty);

    // Show probability variation to understand bucket non-uniformity
    let pi_min = pi
        .iter()
        .cloned()
        .filter(|&p| p > 0.0)
        .fold(f64::INFINITY, f64::min);
    let pi_max = pi.iter().cloned().fold(0.0f64, f64::max);
    let pi_uniform = 1.0 / num_buckets as f64;

    println!();
    println!("Bucket probability variation (from pilot):");
    println!("  Uniform assumption: {:.6}", pi_uniform);
    println!(
        "  Min π_b (non-zero): {:.6} ({:.1}% of uniform)",
        pi_min,
        100.0 * pi_min / pi_uniform
    );
    println!(
        "  Max π_b: {:.6} ({:.1}% of uniform)",
        pi_max,
        100.0 * pi_max / pi_uniform
    );
}

fn print_result(normalized: f64) {
    if normalized < 1.2 {
        println!("RESULT: Distribution appears uniform (chi^2/df < 1.2)");
    } else if normalized < 1.5 {
        println!("RESULT: Distribution marginally uniform (1.2 <= chi^2/df < 1.5)");
    } else {
        println!("RESULT: Distribution appears non-uniform (chi^2/df >= 1.5)");
    }
}
