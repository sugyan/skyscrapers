//! Check if all possible Latin squares can be generated for small n.
//!
//! Known counts:
//! - n=3: 12
//! - n=4: 576
//! - n=5: 161,280

use latin_sampler::{SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashSet;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let max_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let burn_in: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1000);

    let known_counts: &[(usize, usize)] = &[(3, 12), (4, 576), (5, 161_280)];
    let expected = known_counts
        .iter()
        .find(|(size, _)| *size == n)
        .map(|(_, count)| *count);

    println!("=== Latin Square Coverage Test ===");
    println!("n = {}", n);
    if let Some(total) = expected {
        println!("Known total: {}", total);
    }
    println!("max_samples = {}", max_samples);
    println!("burn_in = {}", burn_in);
    println!();

    let params = SamplerParams {
        burn_in,
        ..Default::default()
    };

    let mut unique_squares: HashSet<Vec<u8>> = HashSet::new();
    let start = Instant::now();
    let mut last_new_at = 0;
    let mut actual_samples = 0;

    for seed_idx in 0..max_samples {
        actual_samples = seed_idx + 1;
        let mut rng = ChaCha20Rng::seed_from_u64(seed_idx as u64);
        let sq = sample(n, &mut rng, &params);

        let mut cells = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                cells.push(sq.get(r, c));
            }
        }

        let is_new = unique_squares.insert(cells);
        if is_new {
            last_new_at = seed_idx + 1;
        }

        // Progress report
        if (seed_idx + 1) % 1000 == 0
            || (expected.is_some() && unique_squares.len() == expected.unwrap())
        {
            let elapsed = start.elapsed().as_secs_f64();
            print!("\r[{:6}] unique: {:6}", seed_idx + 1, unique_squares.len());
            if let Some(total) = expected {
                print!(
                    " / {} ({:.1}%)",
                    total,
                    100.0 * unique_squares.len() as f64 / total as f64
                );
            }
            print!(" | {:.1}s", elapsed);
            println!();
        }

        // Stop if we found all
        if let Some(total) = expected {
            if unique_squares.len() == total {
                println!();
                println!(
                    "All {} squares found after {} samples!",
                    total,
                    seed_idx + 1
                );
                break;
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!();
    println!("=== Results ===");
    println!("Unique squares found: {}", unique_squares.len());
    if let Some(total) = expected {
        println!(
            "Coverage: {:.2}%",
            100.0 * unique_squares.len() as f64 / total as f64
        );
    }
    println!("Total samples: {}", actual_samples);
    println!("Last new square found at sample: {}", last_new_at);
    println!("Elapsed time: {:.2}s", elapsed);
}
