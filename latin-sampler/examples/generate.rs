//! Generate a random Latin square with specified size and seed.
//!
//! Usage: cargo run --release --example generate -- <n> [seed]
//!
//! Example:
//!   cargo run --release --example generate -- 8 42

use latin_sampler::{Sampler, SamplerParams};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        eprintln!("Usage: {} <n> [seed]", args[0]);
        std::process::exit(1);
    });

    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let rng = ChaCha20Rng::seed_from_u64(seed);
    let mut sampler = Sampler::new(n, rng, SamplerParams::default());
    let sq = sampler.next().unwrap();

    for r in 0..sq.n() {
        for c in 0..sq.n() {
            print!("{} ", sq.get(r, c));
        }
        println!();
    }
}
