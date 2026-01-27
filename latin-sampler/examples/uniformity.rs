//! Chi-square test on hash buckets for uniformity verification.
//!
//! Instead of checking cell-by-cell frequencies, this hashes the entire
//! Latin square and checks if the distribution across buckets is uniform.

use latin_sampler::{sample, SamplerParams};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let num_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let num_buckets: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(100);
    let burn_in: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(5000);

    println!("=== Latin Square Uniformity Test (Hash Buckets) ===");
    println!("n = {}, samples = {}, buckets = {}, burn_in = {}", n, num_samples, num_buckets, burn_in);
    println!();

    let params = SamplerParams {
        burn_in,
        ..Default::default()
    };

    // Count samples in each bucket
    let mut bucket_counts = vec![0usize; num_buckets];

    for seed_idx in 0..num_samples {
        let mut seed = [0u8; 32];
        seed[0] = (seed_idx & 0xff) as u8;
        seed[1] = ((seed_idx >> 8) & 0xff) as u8;
        seed[2] = ((seed_idx >> 16) & 0xff) as u8;
        seed[3] = ((seed_idx >> 24) & 0xff) as u8;

        let mut rng = ChaCha20Rng::from_seed(seed);
        let sq = sample(n, &mut rng, &params);

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

    let df = num_buckets - 1; // degrees of freedom

    // For chi-square distribution, mean = df, variance = 2*df
    // Normalized chi-square: chi_square / df should be around 1.0 for uniform
    let normalized_chi_sq = chi_square / df as f64;

    println!("Results:");
    println!("  Expected per bucket: {:.2}", expected);
    println!("  Chi-square statistic: {:.2}", chi_square);
    println!("  Degrees of freedom: {}", df);
    println!("  Normalized (χ²/df): {:.4}", normalized_chi_sq);
    println!();

    // Find min/max buckets
    let min_count = *bucket_counts.iter().min().unwrap();
    let max_count = *bucket_counts.iter().max().unwrap();
    println!("  Min bucket count: {} ({:.1}% of expected)", min_count, 100.0 * min_count as f64 / expected);
    println!("  Max bucket count: {} ({:.1}% of expected)", max_count, 100.0 * max_count as f64 / expected);
    println!();

    // Critical values for chi-square (approximate)
    // For df=99: χ²(0.95) ≈ 123.2, χ²(0.99) ≈ 135.8
    // For df=999: χ²(0.95) ≈ 1073.6, χ²(0.99) ≈ 1106.9
    // General rule: if normalized > 1.5, likely non-uniform
    
    if normalized_chi_sq < 1.2 {
        println!("✓ Distribution appears uniform (χ²/df < 1.2)");
    } else if normalized_chi_sq < 1.5 {
        println!("? Distribution marginally uniform (1.2 ≤ χ²/df < 1.5)");
    } else {
        println!("✗ Distribution likely non-uniform (χ²/df ≥ 1.5)");
    }

    // Show bucket distribution histogram
    println!();
    println!("Bucket count distribution:");
    let mut histogram = vec![0usize; 10];
    let bucket_min = (expected * 0.5) as usize;
    let bucket_max = (expected * 1.5) as usize;
    // Ensure a non-zero range; if bucket_max <= bucket_min, fall back to 1.
    let range = if bucket_max > bucket_min {
        (bucket_max - bucket_min + 9) / 10
    } else {
        1
    };
    for &count in &bucket_counts {
        let idx = if count < bucket_min {
            0
        } else if count >= bucket_max {
            9
        } else {
            ((count - bucket_min) / range).min(9)
        };
        histogram[idx] += 1;
    }
    for (i, &h) in histogram.iter().enumerate() {
        let low = bucket_min + i * range;
        let high = low + range;
        let bar = "#".repeat((h * 50 / num_buckets).max(if h > 0 { 1 } else { 0 }));
        println!("  {:3}-{:3}: {:3} {}", low, high, h, bar);
    }
}
