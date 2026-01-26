use latin_sampler::{SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let num_samples: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
    let burn_in: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5000);

    println!("=== Latin Square Sampling Statistics ===");
    println!("n = {}, samples = {}", n, num_samples);
    println!();

    let params = SamplerParams {
        burn_in,
        ..Default::default()
    };
    println!("SamplerParams:");
    println!("  burn_in: {}", params.burn_in);
    println!("  p_do_nothing: {}", params.p_do_nothing);
    println!();

    // Track unique squares and cell frequencies
    let mut unique_squares: HashSet<Vec<u8>> = HashSet::new();
    // freq[r][c][s] = count of symbol s appearing at (r, c)
    let mut freq = vec![vec![vec![0usize; n]; n]; n];

    for seed_idx in 0..num_samples {
        let mut seed = [0u8; 32];
        seed[0] = (seed_idx & 0xff) as u8;
        seed[1] = ((seed_idx >> 8) & 0xff) as u8;
        seed[2] = ((seed_idx >> 16) & 0xff) as u8;
        seed[3] = ((seed_idx >> 24) & 0xff) as u8;

        let mut rng = ChaCha20Rng::from_seed(seed);
        let sq = sample(n, &mut rng, &params);

        // Collect cells for uniqueness check and update frequencies
        let mut cells = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                let s = sq.get(r, c);
                cells.push(s);
                freq[r][c][s as usize] += 1;
            }
        }
        unique_squares.insert(cells);
    }

    println!("Results:");
    println!(
        "  Unique squares: {} / {}",
        unique_squares.len(),
        num_samples
    );
    println!(
        "  Uniqueness rate: {:.2}%",
        100.0 * unique_squares.len() as f64 / num_samples as f64
    );
    println!();

    // Calculate frequency statistics
    // Expected frequency for each symbol at each cell: num_samples / n
    let expected = num_samples as f64 / n as f64;

    // Chi-square statistic for each cell
    let mut total_chi_sq = 0.0;
    let mut max_deviation = 0.0;
    let mut max_dev_cell = (0, 0, 0);

    for r in 0..n {
        for c in 0..n {
            for s in 0..n {
                let observed = freq[r][c][s] as f64;
                let deviation = (observed - expected).abs();
                let chi_sq_contrib = (observed - expected).powi(2) / expected;
                total_chi_sq += chi_sq_contrib;

                if deviation > max_deviation {
                    max_deviation = deviation;
                    max_dev_cell = (r, c, s);
                }
            }
        }
    }

    let num_cells = (n * n * n) as f64;
    let avg_chi_sq = total_chi_sq / num_cells;

    println!("Frequency Analysis:");
    println!("  Expected frequency per symbol per cell: {:.2}", expected);
    println!("  Total chi-square: {:.2}", total_chi_sq);
    println!("  Average chi-square per cell: {:.4}", avg_chi_sq);
    println!(
        "  Max deviation: {:.2} at cell ({}, {}), symbol {}",
        max_deviation, max_dev_cell.0, max_dev_cell.1, max_dev_cell.2
    );
    println!(
        "  Max deviation ratio: {:.2}%",
        100.0 * max_deviation / expected
    );
    println!();

    // Show frequency table for small n
    if n <= 5 {
        println!("Frequency table (symbol counts at each cell):");
        for r in 0..n {
            for c in 0..n {
                print!("  ({},{}):", r, c);
                for s in 0..n {
                    print!(" {}", freq[r][c][s]);
                }
                println!();
            }
        }
    }
}
