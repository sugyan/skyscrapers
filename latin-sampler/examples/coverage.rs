//! Check if all possible Latin squares can be generated for small n.
//!
//! Known counts:
//! - n=3: 12
//! - n=4: 576
//! - n=5: 161,280

use latin_sampler::{Sampler, SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashSet;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let max_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let oneshot_mode = args.iter().any(|s| s == "--oneshot");

    let known_counts: &[(usize, usize)] = &[(3, 12), (4, 576), (5, 161_280)];
    let expected = known_counts
        .iter()
        .find(|(size, _)| *size == n)
        .map(|(_, count)| *count);

    let sampling_mode = if oneshot_mode { "oneshot" } else { "iterator" };
    println!("=== Latin Square Coverage Test ===");
    println!("n = {}, sampling = {}", n, sampling_mode);
    if let Some(total) = expected {
        println!("Known total: {}", total);
    }
    println!("max_samples = {}", max_samples);
    println!("burn_in = n³ = {} (auto)", n * n * n);
    println!();

    let params = SamplerParams::default();

    let mut unique_squares: HashSet<Vec<u8>> = HashSet::new();
    let start = Instant::now();
    let mut last_new_at = 0;

    if oneshot_mode {
        // One-shot mode: call sample() with different seeds for each sample
        for i in 0..max_samples {
            let sample_num = i + 1;
            let mut rng = ChaCha20Rng::seed_from_u64(i as u64);
            let sq = sample(n, &mut rng, &params);
            let cells: Vec<u8> = sq.cells().to_vec();

            let is_new = unique_squares.insert(cells);
            if is_new {
                last_new_at = sample_num;
            }

            // Progress report
            if sample_num % 1000 == 0
                || (expected.is_some() && unique_squares.len() == expected.unwrap())
            {
                let elapsed = start.elapsed().as_secs_f64();
                print!("\r[{:6}] unique: {:6}", sample_num, unique_squares.len());
                if let Some(total) = expected {
                    print!(
                        " / {} ({:.1}%)",
                        total,
                        100.0 * unique_squares.len() as f64 / total as f64
                    );
                }
                print!(" | {:.1}s", elapsed);
                println!();

                // Stop if we found all
                if let Some(total) = expected {
                    if unique_squares.len() == total {
                        println!();
                        println!("All {} squares found after {} samples!", total, sample_num);
                        break;
                    }
                }
            }
        }
    } else {
        // Iterator mode: use Sampler for continuous sampling
        let rng = ChaCha20Rng::seed_from_u64(0);
        let sampler = Sampler::new(n, rng, params);

        for (idx, sq) in sampler.take(max_samples).enumerate() {
            let sample_num = idx + 1;
            let cells: Vec<u8> = sq.cells().to_vec();

            let is_new = unique_squares.insert(cells);
            if is_new {
                last_new_at = sample_num;
            }

            // Progress report
            if sample_num % 1000 == 0
                || (expected.is_some() && unique_squares.len() == expected.unwrap())
            {
                let elapsed = start.elapsed().as_secs_f64();
                print!("\r[{:6}] unique: {:6}", sample_num, unique_squares.len());
                if let Some(total) = expected {
                    print!(
                        " / {} ({:.1}%)",
                        total,
                        100.0 * unique_squares.len() as f64 / total as f64
                    );
                }
                print!(" | {:.1}s", elapsed);
                println!();

                // Stop if we found all
                if let Some(total) = expected {
                    if unique_squares.len() == total {
                        println!();
                        println!("All {} squares found after {} samples!", total, sample_num);
                        break;
                    }
                }
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
    println!("Max samples: {}", max_samples);
    println!("Last new square found at sample: {}", last_new_at);
    println!("Elapsed time: {:.2}s", elapsed);
}
