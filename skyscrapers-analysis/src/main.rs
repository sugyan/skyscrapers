//! Development analysis tools for the Skyscrapers workspace.
//!
//! These are not shipped to end users — they exist to produce performance
//! summaries (e.g. `docs/logic-solver-analysis.md`) when the solver or
//! generator changes in ways that might affect difficulty distributions.

use std::collections::{BTreeMap, BTreeSet};

use clap::{Parser, Subcommand};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use skyscrapers_core::Puzzle;
use skyscrapers_generator::{GeneratorParams, generate};
use skyscrapers_solver::logic::analysis_hooks;
use skyscrapers_solver::logic::difficulty::Technique;
use skyscrapers_solver::{Difficulty, LogicSolver};

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

    /// Measure how often a target difficulty can actually be produced under
    /// a per-seed `max_attempts` budget.
    TargetYield {
        /// Grid size (1-9)
        #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// Target difficulty
        #[arg(short, long)]
        difficulty: Difficulty,

        /// Number of seeds to test (0..samples)
        #[arg(short, long, default_value_t = 100)]
        samples: u64,

        /// Maximum generation attempts per seed
        #[arg(short, long, default_value_t = 200)]
        max_attempts: usize,
    },

    /// For puzzles generated at a target difficulty, measure what changes
    /// when the listed techniques are disabled in the logic solver.
    ///
    /// Records, per puzzle: did baseline rely on the disabled tech (top-level
    /// only — see caveat below), did the puzzle still solve, did the final
    /// difficulty change.
    ///
    /// Caveat: the `baseline_used_disabled` count only counts techniques that
    /// surface as top-level `Step`s in `SolveResult::steps`. Forcing-chain
    /// trials run `propagate()`/`propagate_simple()` internally without
    /// emitting nested steps, so a technique that only fires inside a
    /// forcing-chain trial will not be flagged as "used". The
    /// `still_solved_harder` and `became_unsolvable` counts are unaffected
    /// — they reflect actual outcomes regardless of where the technique
    /// would have run.
    TechniqueNecessity {
        /// Grid size (1-9)
        #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// Target difficulty for the puzzles to test against
        #[arg(short, long)]
        difficulty: Difficulty,

        /// Number of seeds to attempt (0..samples)
        #[arg(short, long, default_value_t = 100)]
        samples: u64,

        /// Maximum generation attempts per seed
        #[arg(long, default_value_t = 500)]
        max_attempts: usize,

        /// Comma-separated technique names to disable, e.g. `NakedSets,DualCluePermutation`
        #[arg(long, value_delimiter = ',', required = true)]
        disable: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::BatchDifficulty { n, seeds } => batch_difficulty(n as usize, seeds),
        Command::TargetYield {
            n,
            difficulty,
            samples,
            max_attempts,
        } => target_yield(n as usize, difficulty, samples, max_attempts),
        Command::TechniqueNecessity {
            n,
            difficulty,
            samples,
            max_attempts,
            disable,
        } => technique_necessity(n as usize, difficulty, samples, max_attempts, &disable),
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

fn target_yield(n: usize, difficulty: Difficulty, samples: u64, max_attempts: usize) {
    let mut succeeded = 0u64;
    let mut failed = 0u64;

    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n)
            .with_target_difficulty(difficulty)
            .with_max_attempts(max_attempts);
        match generate(&mut rng, &params) {
            Ok(_) => {
                succeeded += 1;
                println!("seed={seed:>3}  ok");
            }
            Err(e) => {
                failed += 1;
                println!("seed={seed:>3}  fail  {e}");
            }
        }
    }

    let total = succeeded + failed;
    let rate = if total == 0 {
        0.0
    } else {
        succeeded as f64 / total as f64 * 100.0
    };
    println!(
        "\n=== Summary (n={n}, target={difficulty}, samples={samples}, max_attempts={max_attempts}) ===",
    );
    println!("  succeeded: {succeeded} / {total}  ({rate:.1}%)");
    println!("  failed:    {failed}");
}

fn parse_techniques(names: &[String]) -> Result<Vec<Technique>, String> {
    names
        .iter()
        .map(|s| {
            // Accept the internal enum-style spelling (e.g. `AlsXz`,
            // `XWing`) as well as the displayed public label that the CLI
            // and READMEs use (e.g. `ALS-XZ`, `X-Wing`), case-insensitively.
            // Matching is done after stripping `-` and `_` so users can copy
            // either form without surprise.
            let key: String = s
                .trim()
                .chars()
                .filter(|c| !matches!(c, '-' | '_'))
                .map(|c| c.to_ascii_lowercase())
                .collect();
            match key.as_str() {
                "nakedsingles" => Ok(Technique::NakedSingles),
                "hiddensingles" => Ok(Technique::HiddenSingles),
                "visibilityanalysis" => Ok(Technique::VisibilityAnalysis),
                "nakedsets" => Ok(Technique::NakedSets),
                "xwing" => Ok(Technique::XWing),
                "alsxz" => Ok(Technique::AlsXz),
                "permutationenumeration" => Ok(Technique::PermutationEnumeration),
                "dualcluepermutation" => Ok(Technique::DualCluePermutation),
                "simpleforcingchain" => Ok(Technique::SimpleForcingChain),
                "fullforcingchain" => Ok(Technique::FullForcingChain),
                // CluePruning runs once during SolveState::new and is not
                // routed through the dispatch loop, so analysis_hooks cannot
                // disable it. Reject explicitly to avoid misleading results.
                "cluepruning" => Err(
                    "CluePruning cannot be disabled by this tool (runs outside the dispatch loop)"
                        .to_string(),
                ),
                _ => Err(format!("unknown technique: {:?}", s.trim())),
            }
        })
        .collect()
}

fn solve_baseline(puzzle: &Puzzle) -> (Option<Difficulty>, BTreeSet<Technique>) {
    let res = LogicSolver.solve_with_difficulty(puzzle, 1);
    let techs: BTreeSet<Technique> = res.steps.iter().map(|s| s.technique).collect();
    (res.difficulty, techs)
}

/// RAII guard so the per-thread disabled mask is always cleared, even
/// if `solve_with_difficulty` panics or an early-return is later added.
struct DisabledTechniquesGuard;

impl DisabledTechniquesGuard {
    fn new(disabled: &[Technique]) -> Self {
        analysis_hooks::set_disabled(disabled);
        Self
    }
}

impl Drop for DisabledTechniquesGuard {
    fn drop(&mut self) {
        analysis_hooks::clear_disabled();
    }
}

fn solve_with_disabled(puzzle: &Puzzle, disabled: &[Technique]) -> Option<Difficulty> {
    let _guard = DisabledTechniquesGuard::new(disabled);
    LogicSolver.solve_with_difficulty(puzzle, 1).difficulty
}

fn technique_necessity(
    n: usize,
    difficulty: Difficulty,
    samples: u64,
    max_attempts: usize,
    disable: &[String],
) {
    let disabled = match parse_techniques(disable) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    let mut gen_failed = 0u64;
    let mut puzzles_tested = 0u64;
    let mut baseline_used = 0u64; // baseline used at least one disabled tech
    let mut still_solved_same = 0u64;
    let mut still_solved_easier = 0u64;
    let mut still_solved_harder = 0u64;
    let mut became_unsolvable = 0u64;

    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n)
            .with_target_difficulty(difficulty)
            .with_max_attempts(max_attempts);
        let (puzzle, _sol) = match generate(&mut rng, &params) {
            Ok(v) => v,
            Err(_) => {
                gen_failed += 1;
                println!("seed={seed:>3}  gen_fail");
                continue;
            }
        };

        puzzles_tested += 1;
        let (base_diff, base_techs) = solve_baseline(&puzzle);
        let used = disabled.iter().any(|t| base_techs.contains(t));
        if used {
            baseline_used += 1;
        }

        let new_diff = solve_with_disabled(&puzzle, &disabled);

        let label = match (base_diff, new_diff) {
            (Some(b), Some(a)) if a == b => {
                still_solved_same += 1;
                format!("same {b}")
            }
            (Some(b), Some(a)) if a < b => {
                still_solved_easier += 1;
                format!("easier {b}->{a}")
            }
            (Some(b), Some(a)) => {
                still_solved_harder += 1;
                format!("harder {b}->{a}")
            }
            (Some(b), None) => {
                became_unsolvable += 1;
                format!("unsolvable (was {b})")
            }
            (None, _) => "baseline_unsolvable".into(),
        };
        println!(
            "seed={seed:>3}  used={}  {}",
            if used { "yes" } else { "no " },
            label
        );
    }

    let disabled_str: Vec<String> = disabled.iter().map(|t| format!("{t:?}")).collect();
    println!(
        "\n=== Summary (n={n}, target={difficulty}, samples={samples}, disable=[{}]) ===",
        disabled_str.join(", ")
    );
    println!("  gen_failed:           {gen_failed}");
    println!("  puzzles_tested:       {puzzles_tested}");
    println!("  baseline_used_disabled: {baseline_used}");
    println!("  still_solved_same:    {still_solved_same}");
    println!("  still_solved_easier:  {still_solved_easier}");
    println!("  still_solved_harder:  {still_solved_harder}");
    println!("  became_unsolvable:    {became_unsolvable}");
}
