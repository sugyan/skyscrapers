//! Development analysis tools for the Skyscrapers workspace.
//!
//! These are not shipped to end users — they exist to produce performance
//! summaries (e.g. `docs/logic-solver-analysis.md`) when the solver or
//! generator changes in ways that might affect difficulty distributions.

use std::collections::{BTreeMap, BTreeSet};

use clap::{Parser, Subcommand};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use skyscrapers_generator::{GeneratorParams, generate};
use skyscrapers_solver::LogicSolver;
use skyscrapers_solver::logic::difficulty::Technique;

#[derive(Parser)]
#[command(
    name = "skyscrapers-analysis",
    about = "Development analysis tools for the Skyscrapers workspace"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate puzzles across a seed range and summarize logic-solver difficulty.
    BatchDifficulty {
        /// Grid size (1-9)
        #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// Number of seeds to test (0..seeds)
        #[arg(short, long, default_value_t = 100)]
        seeds: u64,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::BatchDifficulty { n, seeds } => batch_difficulty(n as usize, seeds),
    }
}

fn batch_difficulty(n: usize, seeds: u64) {
    let mut counts: BTreeMap<String, usize> = Default::default();
    let mut unsolved = 0usize;
    let mut tech_puzzles: BTreeMap<Technique, usize> = Default::default();
    let mut tech_steps: BTreeMap<Technique, usize> = Default::default();

    for seed in 0..seeds {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n);
        let (puzzle, _sol) = match generate(&mut rng, &params) {
            Ok(v) => v,
            Err(_) => {
                println!("seed={seed:>3}  gen_err");
                continue;
            }
        };
        let res = LogicSolver.solve_with_difficulty(&puzzle, 1);
        let techs: BTreeSet<Technique> = res.steps.iter().map(|s| s.technique).collect();
        for t in &techs {
            *tech_puzzles.entry(*t).or_default() += 1;
        }
        for step in &res.steps {
            *tech_steps.entry(step.technique).or_default() += 1;
        }
        match res.difficulty {
            Some(d) => {
                *counts.entry(d.to_string()).or_default() += 1;
                let tech_str: Vec<String> = techs.iter().map(|t| format!("{t:?}")).collect();
                println!(
                    "seed={:>3}  yes  {:<11}  {}",
                    seed,
                    d.to_string(),
                    tech_str.join(", ")
                );
            }
            None => {
                unsolved += 1;
                println!("seed={seed:>3}  no");
            }
        }
    }

    println!("\n=== Summary (n={n}, {seeds} seeds) ===");
    for (d, c) in &counts {
        println!("  {d}: {c}");
    }
    println!("  unsolved: {unsolved}");

    println!("\n=== Technique: puzzles in which it appears ===");
    let mut v: Vec<_> = tech_puzzles.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for (t, c) in v {
        println!("  {t:?}: {c}");
    }

    println!("\n=== Technique: total step count ===");
    let mut v: Vec<_> = tech_steps.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for (t, c) in v {
        println!("  {t:?}: {c}");
    }
}
