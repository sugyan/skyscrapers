use std::io::Read;
use std::process;

use clap::{Parser, Subcommand};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use skyscrapers_core::{Clues, Puzzle};
use skyscrapers_generator::{GeneratorParams, generate};
use skyscrapers_solver::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};
use skyscrapers_solver::{BacktrackingSolver, LogicSolver, Solver};

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

        /// Use the logic solver and print a step-by-step reasoning trace.
        #[arg(long)]
        logic: bool,
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
        Command::Solve { file, logic } => {
            let input = read_input(file.as_deref());
            let puzzle: Puzzle = input.parse().unwrap_or_else(|e| {
                eprintln!("error: failed to parse puzzle: {e}");
                process::exit(1);
            });

            if logic {
                solve_logic(&puzzle);
            } else {
                solve_backtracking(&puzzle);
            }
        }
    }
}

fn read_input(file: Option<&str>) -> String {
    match file {
        Some(path) => std::fs::read_to_string(path).unwrap_or_else(|e| {
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
    }
}

fn solve_backtracking(puzzle: &Puzzle) {
    let solutions = BacktrackingSolver.solve(puzzle, 2);
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

fn solve_logic(puzzle: &Puzzle) {
    let result = LogicSolver.solve_with_difficulty(puzzle, 1);

    for step in &result.steps {
        println!("{}", format_step(step, puzzle));
    }

    match (result.solutions.first(), result.difficulty) {
        (Some(sol), Some(diff)) => {
            println!("Difficulty: {diff:?} ({} steps)", result.steps.len());
            println!("{sol}");
        }
        _ => {
            println!(
                "Logic solver could not solve this puzzle (stuck after {} steps).",
                result.steps.len()
            );
            process::exit(1);
        }
    }
}

/// Format one trace line: `[Technique] <actions>  (<reason>)`.
fn format_step(step: &Step, puzzle: &Puzzle) -> String {
    let tech = technique_name(step.technique);
    let has_place = step
        .actions
        .iter()
        .any(|a| matches!(a, Action::Place { .. }));

    if has_place {
        // Placement step: show "Rx Cy = v (reason)" per Place action.
        // Typically there is a single Place; join with ", " if somehow more.
        let places: Vec<String> = step
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::Place { row, col, value } => {
                    Some(format!("{} = {}", cell_ref(*row, *col), value))
                }
                _ => None,
            })
            .collect();
        format!(
            "[{tech}] {}  ({})",
            places.join(", "),
            format_reason(&step.reason, puzzle),
        )
    } else {
        // Elimination-only step: reason first, then "-> -v RxCy, ..."
        let elims: Vec<String> = step
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::Eliminate { row, col, value } => {
                    Some(format!("-{value} {}", cell_ref(*row, *col)))
                }
                _ => None,
            })
            .collect();
        format!(
            "[{tech}] {}  ->  {}",
            format_reason(&step.reason, puzzle),
            elims.join(", "),
        )
    }
}

fn technique_name(t: Technique) -> &'static str {
    match t {
        Technique::NakedSingles => "NakedSingles",
        Technique::HiddenSingles => "HiddenSingles",
        Technique::CluePruning => "CluePruning",
        Technique::NakedSets => "NakedSets",
        Technique::HiddenSets => "HiddenSets",
        Technique::XWing => "XWing",
        Technique::XYWing => "XYWing",
        Technique::WWing => "WWing",
        Technique::AlsXz => "ALS-XZ",
        Technique::PermutationEnumeration => "PermutationEnumeration",
        Technique::DualCluePermutation => "DualCluePermutation",
        Technique::SimpleForcingChain => "SimpleForcingChain",
        Technique::FullForcingChain => "FullForcingChain",
    }
}

fn cell_ref(row: usize, col: usize) -> String {
    format!("R{}C{}", row + 1, col + 1)
}

fn cells_set(cells: &[(usize, usize)]) -> String {
    let inner: Vec<String> = cells.iter().map(|&(r, c)| cell_ref(r, c)).collect();
    format!("{{{}}}", inner.join(","))
}

fn line_name(line: Line) -> String {
    match line {
        Line::Row(r) => format!("Row {}", r + 1),
        Line::Col(c) => format!("Col {}", c + 1),
    }
}

fn lines_list(lines: &[Line]) -> String {
    let inner: Vec<String> = lines.iter().copied().map(line_name).collect();
    format!("[{}]", inner.join(","))
}

fn values_set(values: &[u8]) -> String {
    let inner: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    format!("{{{}}}", inner.join(","))
}

/// Render a clue with its value, e.g. `Right 1=2`.
fn clue_with_value(clue: CluePosition, clues: &Clues) -> String {
    let (label, idx, value) = match clue {
        CluePosition::Top(i) => ("Top", i, clues.top(i)),
        CluePosition::Bottom(i) => ("Bottom", i, clues.bottom(i)),
        CluePosition::Left(i) => ("Left", i, clues.left(i)),
        CluePosition::Right(i) => ("Right", i, clues.right(i)),
    };
    match value {
        Some(v) => format!("{label} {}={}", idx + 1, v),
        None => format!("{label} {}", idx + 1),
    }
}

fn format_reason(reason: &Reason, puzzle: &Puzzle) -> String {
    match reason {
        Reason::SingleCandidate { row, col } => {
            format!("only candidate at {}", cell_ref(*row, *col))
        }
        Reason::UniqueInLine { line, value } => {
            format!("{value} is unique in {}", line_name(*line))
        }
        Reason::ClueConstraint { clue, expected } => {
            format!(
                "clue constraint ({}, expected={expected})",
                clue_with_value(*clue, &puzzle.clues),
            )
        }
        Reason::SetInLine {
            line,
            cells,
            values,
        } => format!(
            "{}: {} locked in {}",
            line_name(*line),
            values_set(values),
            cells_set(cells),
        ),
        Reason::FishPattern {
            value,
            base_lines,
            cover_lines,
        } => format!(
            "{value} in base {} -> cover {}",
            lines_list(base_lines),
            lines_list(cover_lines),
        ),
        Reason::PermutationElimination { line, clue } => format!(
            "{} ({})",
            line_name(*line),
            clue_with_value(*clue, &puzzle.clues),
        ),
        Reason::DualCluePermutationElimination {
            line,
            clue_a,
            clue_b,
        } => format!(
            "{} ({}, {})",
            line_name(*line),
            clue_with_value(*clue_a, &puzzle.clues),
            clue_with_value(*clue_b, &puzzle.clues),
        ),
        Reason::XYWingElimination {
            pivot,
            wing_a,
            wing_b,
            eliminated_value,
        } => format!(
            "XY-Wing pivot={} wings={},{} eliminates {eliminated_value}",
            cell_ref(pivot.0, pivot.1),
            cell_ref(wing_a.0, wing_a.1),
            cell_ref(wing_b.0, wing_b.1),
        ),
        Reason::WWingElimination {
            cell_a,
            cell_b,
            link_c,
            link_d,
            link_value,
            eliminated_value,
        } => format!(
            "W-Wing {},{} via link {},{}={link_value} eliminates {eliminated_value}",
            cell_ref(cell_a.0, cell_a.1),
            cell_ref(cell_b.0, cell_b.1),
            cell_ref(link_c.0, link_c.1),
            cell_ref(link_d.0, link_d.1),
        ),
        Reason::AlsXzElimination {
            als_a,
            als_b,
            rcc_value,
            eliminated_value,
        } => format!(
            "ALS-XZ {}+{} rcc={rcc_value} eliminates {eliminated_value}",
            cells_set(als_a),
            cells_set(als_b),
        ),
        Reason::ForcingChainElimination {
            assumed_cell,
            assumed_value,
        } => format!(
            "assume {}={assumed_value} -> contradiction",
            cell_ref(assumed_cell.0, assumed_cell.1),
        ),
    }
}
