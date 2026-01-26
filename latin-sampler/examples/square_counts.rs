//! Count exact occurrences of each Latin square for small n.

use latin_sampler::{sample, SamplerParams};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let num_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let burn_in: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5000);

    println!("=== Latin Square Exact Count Test ===");
    println!("n = {}, samples = {}, burn_in = {}", n, num_samples, burn_in);
    println!();

    let params = SamplerParams {
        burn_in,
        ..Default::default()
    };

    // Count each unique square
    let mut counts: HashMap<Vec<u8>, usize> = HashMap::new();

    for seed_idx in 0..num_samples {
        let mut seed = [0u8; 32];
        seed[0] = (seed_idx & 0xff) as u8;
        seed[1] = ((seed_idx >> 8) & 0xff) as u8;
        seed[2] = ((seed_idx >> 16) & 0xff) as u8;
        seed[3] = ((seed_idx >> 24) & 0xff) as u8;

        let mut rng = ChaCha20Rng::from_seed(seed);
        let sq = sample(n, &mut rng, &params);

        let mut cells = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                cells.push(sq.get(r, c));
            }
        }
        *counts.entry(cells).or_insert(0) += 1;
    }

    let num_unique = counts.len();
    let expected = num_samples as f64 / num_unique as f64;
    
    // Chi-square test on actual counts
    let chi_square: f64 = counts
        .values()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum();
    
    let df = num_unique - 1;
    let normalized = chi_square / df as f64;

    println!("Results:");
    println!("  Unique squares: {}", num_unique);
    println!("  Expected per square: {:.2}", expected);
    println!("  Chi-square: {:.2}", chi_square);
    println!("  Degrees of freedom: {}", df);
    println!("  Normalized (χ²/df): {:.4}", normalized);
    println!();

    // Count distribution
    let mut count_values: Vec<usize> = counts.values().copied().collect();
    count_values.sort();
    
    let min_count = *count_values.first().unwrap();
    let max_count = *count_values.last().unwrap();
    let median = count_values[count_values.len() / 2];
    
    println!("  Min occurrences: {}", min_count);
    println!("  Median occurrences: {}", median);
    println!("  Max occurrences: {}", max_count);
    println!();

    // Histogram of counts
    println!("Occurrence distribution:");
    let mut histogram: HashMap<usize, usize> = HashMap::new();
    for &c in &count_values {
        *histogram.entry(c).or_insert(0) += 1;
    }
    let mut hist_keys: Vec<usize> = histogram.keys().copied().collect();
    hist_keys.sort();
    for k in hist_keys {
        let v = histogram[&k];
        let bar = "#".repeat((v * 50 / num_unique).max(if v > 0 { 1 } else { 0 }));
        println!("  {:3} times: {:4} squares {}", k, v, bar);
    }

    println!();
    if normalized < 1.2 {
        println!("✓ Distribution appears uniform");
    } else if normalized < 1.5 {
        println!("? Distribution marginally uniform");
    } else {
        println!("✗ Distribution likely non-uniform");
    }
}
