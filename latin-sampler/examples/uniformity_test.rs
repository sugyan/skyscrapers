//! Improved uniformity test with correct chi-square methodology.
//!
//! Key improvements over existing tests:
//! - n=4: Uses known total of 576 Latin squares as denominator (not observed unique count)
//! - n>=5: Uses larger bucket counts (4096 for n=5, 8192 for n>=6)
//! - Includes unobserved categories in chi-square calculation for n=4

use latin_sampler::{SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};

/// Known number of Latin squares for small n
const LATIN_SQUARES_N4: usize = 576;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let num_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100_000);

    println!("=== Improved Uniformity Test ===");
    println!(
        "n = {}, samples = {}, burn_in = n³ = {} (auto)",
        n,
        num_samples,
        n * n * n
    );
    println!();

    let params = SamplerParams::default();

    if n == 4 {
        run_n4_direct_test(n, num_samples, &params);
    } else {
        let num_buckets = if n == 5 { 4096 } else { 8192 };
        run_hash_bucket_test(n, num_samples, num_buckets, &params);
    }
}

/// Direct count test for n=4 using the known total of 576 Latin squares.
fn run_n4_direct_test(n: usize, num_samples: usize, params: &SamplerParams) {
    println!("Mode: Direct count ({} categories)", LATIN_SQUARES_N4);
    println!();

    // Count each unique square
    let mut counts: HashMap<Vec<u8>, usize> = HashMap::new();

    for seed_idx in 0..num_samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed_idx as u64);
        let sq = sample(n, &mut rng, params);

        let mut cells = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                cells.push(sq.get(r, c));
            }
        }
        *counts.entry(cells).or_insert(0) += 1;
    }

    let num_unique = counts.len();

    // Expected value based on KNOWN total (576), not observed unique count
    let expected = num_samples as f64 / LATIN_SQUARES_N4 as f64;

    // Chi-square calculation including unobserved categories
    // Observed categories: sum of (count - expected)^2 / expected
    let chi_sq_observed: f64 = counts
        .values()
        .map(|&c| {
            let diff = c as f64 - expected;
            diff * diff / expected
        })
        .sum();

    // Unobserved categories: (0 - expected)^2 / expected = expected
    let num_unobserved = LATIN_SQUARES_N4 - num_unique;
    let chi_sq_unobserved = num_unobserved as f64 * expected;

    let chi_square = chi_sq_observed + chi_sq_unobserved;
    let df = LATIN_SQUARES_N4 - 1; // 575
    let normalized = chi_square / df as f64;

    println!("Results:");
    println!(
        "  Unique squares observed: {} / {}",
        num_unique, LATIN_SQUARES_N4
    );
    println!("  Expected per category: {:.2}", expected);
    println!("  Chi-square: {:.2}", chi_square);
    println!("    (from observed: {:.2})", chi_sq_observed);
    println!("    (from unobserved: {:.2})", chi_sq_unobserved);
    println!("  Degrees of freedom: {}", df);
    println!("  Normalized (chi^2/df): {:.4}", normalized);
    println!();

    print_result(normalized);

    // Additional statistics
    if !counts.is_empty() {
        let mut count_values: Vec<usize> = counts.values().copied().collect();
        count_values.sort();

        let min_count = *count_values.first().unwrap();
        let max_count = *count_values.last().unwrap();

        println!();
        println!("Observed square statistics:");
        println!(
            "  Min occurrences: {} ({:.1}% of expected)",
            min_count,
            100.0 * min_count as f64 / expected
        );
        println!(
            "  Max occurrences: {} ({:.1}% of expected)",
            max_count,
            100.0 * max_count as f64 / expected
        );
    }
}

/// Hash bucket test for n>=5 with configurable bucket count.
fn run_hash_bucket_test(n: usize, num_samples: usize, num_buckets: usize, params: &SamplerParams) {
    println!("Mode: Hash bucket ({} buckets)", num_buckets);
    println!();

    let mut bucket_counts = vec![0usize; num_buckets];

    for seed_idx in 0..num_samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed_idx as u64);
        let sq = sample(n, &mut rng, params);

        // Hash the entire square
        let mut hasher = DefaultHasher::new();
        for r in 0..n {
            for c in 0..n {
                sq.get(r, c).hash(&mut hasher);
            }
        }
        let hash = hasher.finish();
        let bucket = (hash as usize) % num_buckets;
        bucket_counts[bucket] += 1;
    }

    // Chi-square test
    let expected = num_samples as f64 / num_buckets as f64;
    let chi_square: f64 = bucket_counts
        .iter()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum();

    let df = num_buckets - 1;
    let normalized = chi_square / df as f64;

    println!("Results:");
    println!("  Expected per bucket: {:.2}", expected);
    println!("  Chi-square: {:.2}", chi_square);
    println!("  Degrees of freedom: {}", df);
    println!("  Normalized (chi^2/df): {:.4}", normalized);
    println!();

    print_result(normalized);

    // Bucket statistics
    let min_count = *bucket_counts.iter().min().unwrap();
    let max_count = *bucket_counts.iter().max().unwrap();
    let empty_buckets = bucket_counts.iter().filter(|&&c| c == 0).count();

    println!();
    println!("Bucket statistics:");
    println!(
        "  Min bucket count: {} ({:.1}% of expected)",
        min_count,
        100.0 * min_count as f64 / expected
    );
    println!(
        "  Max bucket count: {} ({:.1}% of expected)",
        max_count,
        100.0 * max_count as f64 / expected
    );
    println!("  Empty buckets: {}", empty_buckets);
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
