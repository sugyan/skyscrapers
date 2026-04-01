use std::io::Read;
use std::process;

use clap::{Parser, Subcommand};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use skyscrapers_core::Puzzle;
use skyscrapers_generator::{GeneratorParams, generate};
use skyscrapers_solver::{BacktrackingSolver, Solver};

#[derive(Parser)]
#[command(
    name = "skyscrapers",
    about = "Skyscrapers puzzle generator and solver"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a Skyscrapers puzzle
    Generate {
        /// Grid size (1-9)
        #[arg(short, default_value_t = 7, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// RNG seed (random if omitted)
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Solve a Skyscrapers puzzle
    Solve {
        /// Puzzle file (reads stdin if omitted)
        file: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate { n, seed } => {
            let n = n as usize;
            let seed = seed.unwrap_or_else(|| {
                let s = rand::random::<u64>();
                eprintln!("seed: {s}");
                s
            });
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let params = GeneratorParams::new(n, BacktrackingSolver);
            let (puzzle, _solution) = generate(&mut rng, &params);
            println!("{puzzle}");
        }
        Command::Solve { file } => {
            let input = match file {
                Some(path) => std::fs::read_to_string(&path).unwrap_or_else(|e| {
                    eprintln!("error: cannot read {path}: {e}");
                    process::exit(1);
                }),
                None => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .unwrap_or_else(|e| {
                            eprintln!("error: cannot read stdin: {e}");
                            process::exit(1);
                        });
                    buf
                }
            };

            let puzzle: Puzzle = input.parse().unwrap_or_else(|e| {
                eprintln!("error: failed to parse puzzle: {e}");
                process::exit(1);
            });

            let solutions = BacktrackingSolver.solve(&puzzle, 2);
            match solutions.len() {
                0 => {
                    eprintln!("error: no solution found");
                    process::exit(1);
                }
                1 => {
                    println!("{}", solutions[0]);
                }
                _ => {
                    eprintln!("error: multiple solutions found (not a valid puzzle)");
                    process::exit(1);
                }
            }
        }
    }
}
