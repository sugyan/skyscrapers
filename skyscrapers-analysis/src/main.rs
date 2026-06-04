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
use skyscrapers_solver::logic::difficulty::{Action, Line, Reason, Step, Technique};
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

    /// Reproduce a single puzzle (by seed/n/difficulty, same as the CLI
    /// `generate`) and print its logic-solver trace and difficulty, optionally
    /// with some techniques disabled. Useful for asking "what solves this
    /// puzzle if technique X is taken away?".
    Explain {
        /// Grid size (1-9)
        #[arg(short, default_value_t = 7, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// RNG seed (must match the seed used with the CLI `generate`)
        #[arg(long)]
        seed: u64,

        /// Target difficulty (must match what was used with `generate`)
        #[arg(long)]
        difficulty: Option<Difficulty>,

        /// Comma-separated technique names to disable, e.g. `XyChain`
        #[arg(long, value_delimiter = ',')]
        disable: Vec<String>,
    },

    /// Run the full analysis suite and emit the data-driven sections of
    /// `docs/logic-solver-analysis.md` as Markdown (Target Yield, Technique
    /// Necessity, Batch Test Results, Technique Usage). Per-seed dumps and
    /// the interpretive prose are intentionally not generated. Redirect to
    /// refresh the doc's tables in one reproducible command.
    Report {
        /// Number of seeds per cell (must be >= 1).
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u64).range(1..))]
        samples: u64,

        /// Max generation attempts per seed for target-driven runs.
        #[arg(long, default_value_t = 300)]
        yield_attempts: usize,

        /// Max generation attempts per seed for technique-necessity runs.
        #[arg(long, default_value_t = 500)]
        necessity_attempts: usize,
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
        Command::Explain {
            n,
            seed,
            difficulty,
            disable,
        } => explain(n as usize, seed, difficulty, &disable),
        Command::Report {
            samples,
            yield_attempts,
            necessity_attempts,
        } => report(samples, yield_attempts, necessity_attempts),
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
                "xychain" => Ok(Technique::XyChain),
                "alsxz" => Ok(Technique::AlsXz),
                "permutationenumeration" => Ok(Technique::PermutationEnumeration),
                "simplepermutation" => Ok(Technique::SimplePermutation),
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

/// Difficulty tiers in ascending order, for table columns.
const ALL_DIFFICULTIES: [Difficulty; 5] = [
    Difficulty::Easy,
    Difficulty::Medium,
    Difficulty::Hard,
    Difficulty::Expert,
    Difficulty::Master,
];

/// Techniques in a stable display order for the usage tables.
const TECHNIQUE_ROWS: [Technique; 13] = [
    Technique::NakedSingles,
    Technique::CluePruning,
    Technique::HiddenSingles,
    Technique::SimplePermutation,
    Technique::VisibilityAnalysis,
    Technique::PermutationEnumeration,
    Technique::NakedSets,
    Technique::XyChain,
    Technique::AlsXz,
    Technique::XWing,
    Technique::SimpleForcingChain,
    Technique::FullForcingChain,
    Technique::DualCluePermutation,
];

/// Unseeded batch aggregates for one size (no per-seed records — `report`
/// only needs the totals). Mirrors the generation used by `batch_difficulty`.
struct BatchTotals {
    counts: BTreeMap<Difficulty, usize>,
    unsolved: usize,
    tech_puzzles: BTreeMap<Technique, usize>,
    tech_steps: BTreeMap<Technique, usize>,
}

fn report_batch(n: usize, samples: u64) -> BatchTotals {
    let mut totals = BatchTotals {
        counts: Default::default(),
        unsolved: 0,
        tech_puzzles: Default::default(),
        tech_steps: Default::default(),
    };
    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n);
        let Ok((puzzle, _sol)) = generate(&mut rng, &params) else {
            continue;
        };
        let res = LogicSolver.solve_with_difficulty(&puzzle, 1);
        let techs: BTreeSet<Technique> = res.steps.iter().map(|s| s.technique).collect();
        for t in &techs {
            *totals.tech_puzzles.entry(*t).or_default() += 1;
        }
        for step in &res.steps {
            *totals.tech_steps.entry(step.technique).or_default() += 1;
        }
        match res.difficulty {
            Some(d) => *totals.counts.entry(d).or_default() += 1,
            None => totals.unsolved += 1,
        }
    }
    totals
}

/// Number of seeds for which the generator reached the target difficulty.
fn report_yield(n: usize, difficulty: Difficulty, samples: u64, max_attempts: usize) -> u64 {
    (0..samples)
        .filter(|seed| {
            let mut rng = ChaCha20Rng::seed_from_u64(*seed);
            let params = GeneratorParams::new(n)
                .with_target_difficulty(difficulty)
                .with_max_attempts(max_attempts);
            generate(&mut rng, &params).is_ok()
        })
        .count() as u64
}

/// `(tested, used, harder, unsolvable)` counts for disabling `disabled` on
/// puzzles generated at `difficulty`. Mirrors `technique_necessity` without
/// printing. Seeds whose target-driven generation fails are skipped, so
/// `tested` may be < `samples` (the effective denominator for the other
/// counts); the caller surfaces this.
fn report_necessity(
    n: usize,
    difficulty: Difficulty,
    samples: u64,
    max_attempts: usize,
    disabled: &[Technique],
) -> (u64, u64, u64, u64) {
    let (mut tested, mut used, mut harder, mut unsolvable) = (0u64, 0u64, 0u64, 0u64);
    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n)
            .with_target_difficulty(difficulty)
            .with_max_attempts(max_attempts);
        let Ok((puzzle, _sol)) = generate(&mut rng, &params) else {
            continue;
        };
        tested += 1;
        let (base_diff, base_techs) = solve_baseline(&puzzle);
        if disabled.iter().any(|t| base_techs.contains(t)) {
            used += 1;
        }
        match (base_diff, solve_with_disabled(&puzzle, disabled)) {
            (Some(b), Some(a)) if a > b => harder += 1,
            (Some(_), None) => unsolvable += 1,
            _ => {}
        }
    }
    (tested, used, harder, unsolvable)
}

fn report(samples: u64, yield_attempts: usize, necessity_attempts: usize) {
    let sizes = [4usize, 5, 6, 7];
    // Generation/solve is fast for small n but the n=7 target-driven runs are
    // the slow part; surface progress on stderr so a redirected stdout stays
    // clean Markdown.
    eprintln!("running batch ({} seeds/size)…", samples);
    let batches: Vec<(usize, BatchTotals)> = sizes
        .iter()
        .map(|&n| (n, report_batch(n, samples)))
        .collect();

    println!(
        "## Target Yield (seeds 0-{}, {} seeds per (size, target))\n",
        samples - 1,
        samples
    );
    println!(
        "Generator success rate when a target difficulty is requested with\n`max_attempts={yield_attempts}` per seed.\n"
    );
    println!("| n | easy | medium | hard | expert | master |");
    println!("|---|------|--------|------|--------|--------|");
    for &n in &sizes {
        eprintln!("yield n={n}…");
        let cells: Vec<String> = ALL_DIFFICULTIES
            .iter()
            .map(|&d| report_yield(n, d, samples, yield_attempts).to_string())
            .collect();
        println!("| {n} | {} |", cells.join(" | "));
    }

    println!("\n## Technique Necessity (target-driven, {samples} seeds per cell)\n");
    println!(
        "Each cell shows `used / harder / unsolvable` for puzzles generated at\nthe target difficulty and re-solved with the technique disabled\n(`max_attempts={necessity_attempts}`). Counts are over the seeds that\nsuccessfully generated a puzzle at the target — failed seeds are skipped,\nso the per-cell denominator is the matching Target Yield above (every\nseed, for the tiers shown here).\n"
    );
    let nec_sizes = [5usize, 6, 7];
    let nec_tiers = [Difficulty::Hard, Difficulty::Expert, Difficulty::Master];
    let mut nec_genfail = 0u64;
    for tech in [
        Technique::XyChain,
        Technique::AlsXz,
        Technique::DualCluePermutation,
    ] {
        println!("### Disable {tech:?}\n");
        println!("| n | hard | expert | master |");
        println!("|---|------|--------|--------|");
        for &n in &nec_sizes {
            eprintln!("necessity {tech:?} n={n}…");
            let cells: Vec<String> = nec_tiers
                .iter()
                .map(|&d| {
                    let (tested, u, h, x) =
                        report_necessity(n, d, samples, necessity_attempts, &[tech]);
                    nec_genfail += samples - tested;
                    format!("{u}/{h}/{x}")
                })
                .collect();
            println!("| {n} | {} |", cells.join(" | "));
        }
        println!();
    }
    if nec_genfail > 0 {
        println!(
            "> Note: {nec_genfail} seed(s) failed target generation and were\n> excluded, so those cells are out of fewer than {samples} seeds.\n"
        );
    }

    println!(
        "## Batch Test Results (seeds 0-{}, {} seeds per size)\n",
        samples - 1,
        samples
    );
    println!("| n | Easy | Medium | Hard | Expert | Master | Unsolved | Success |");
    println!("|---|------|--------|------|--------|--------|----------|---------|");
    for (n, b) in &batches {
        let tier_cells: Vec<String> = ALL_DIFFICULTIES
            .iter()
            .map(|d| b.counts.get(d).copied().unwrap_or(0).to_string())
            .collect();
        let solved = samples as usize - b.unsolved;
        let pct = solved as f64 / samples as f64 * 100.0;
        println!(
            "| {n} | {} | {} | {:.0}% |",
            tier_cells.join(" | "),
            b.unsolved,
            pct
        );
    }

    print_usage_table("total step count", samples, &batches, &sizes, |b| {
        &b.tech_steps
    });
    print_usage_table(
        "puzzles in which it appears",
        samples,
        &batches,
        &sizes,
        |b| &b.tech_puzzles,
    );
    eprintln!("done.");
}

fn print_usage_table(
    title: &str,
    samples: u64,
    batches: &[(usize, BatchTotals)],
    sizes: &[usize],
    pick: impl Fn(&BatchTotals) -> &BTreeMap<Technique, usize>,
) {
    println!("\n## Technique Usage ({title} across {samples} seeds per size)\n");
    let header: Vec<String> = sizes.iter().map(|n| format!("n={n}")).collect();
    println!("| Technique | {} |", header.join(" | "));
    println!("|-----------|{}", "-----|".repeat(sizes.len()));
    for tech in TECHNIQUE_ROWS {
        let cells: Vec<String> = batches
            .iter()
            .map(|(_, b)| match pick(b).get(&tech).copied().unwrap_or(0) {
                0 => "—".to_string(),
                v => v.to_string(),
            })
            .collect();
        println!("| {tech:?} | {} |", cells.join(" | "));
    }
}

fn explain(n: usize, seed: u64, difficulty: Option<Difficulty>, disable: &[String]) {
    let disabled = match parse_techniques(disable) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    // Reproduce the puzzle exactly as `skyscrapers-cli generate` would.
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut params = GeneratorParams::new(n);
    if let Some(d) = difficulty {
        params = params.with_target_difficulty(d);
    }
    let (puzzle, _sol) = match generate(&mut rng, &params) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: generation failed: {e}");
            std::process::exit(1);
        }
    };

    println!("{puzzle}");
    let disabled_str: Vec<String> = disabled.iter().map(|t| format!("{t:?}")).collect();
    println!(
        "\n=== Logic trace (n={n}, seed={seed}, disable=[{}]) ===",
        disabled_str.join(", ")
    );

    let result = {
        let _guard = DisabledTechniquesGuard::new(&disabled);
        LogicSolver.solve_with_difficulty(&puzzle, 1)
    };

    // Per-technique step tally and the longest XY-Chain seen, so the trade-off
    // between "removed XY-Chain" and "what replaced it" is visible at a glance.
    let mut tech_steps: BTreeMap<Technique, usize> = Default::default();
    let mut max_xy_len = 0usize;
    for step in &result.steps {
        *tech_steps.entry(step.technique).or_default() += 1;
        if let Reason::XyChainElimination { chain, .. } = &step.reason {
            max_xy_len = max_xy_len.max(chain.len());
        }
        println!("{}", format_step(step));
    }

    println!("\n=== Summary ===");
    match result.difficulty {
        Some(d) => println!("  difficulty: {d}  ({} steps)", result.steps.len()),
        None => println!(
            "  difficulty: UNSOLVABLE  (stuck after {} steps)",
            result.steps.len()
        ),
    }
    if max_xy_len > 0 {
        println!("  longest XY-Chain: {max_xy_len} cells");
    }
    println!("  technique step counts:");
    let mut v: Vec<_> = tech_steps.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for (t, c) in v {
        println!("    {t:?}: {c}");
    }
}

/// 1-based `RrCc` cell label, matching the CLI trace's convention.
fn cell_label(row: usize, col: usize) -> String {
    format!("R{}C{}", row + 1, col + 1)
}

fn line_label(line: &Line) -> String {
    match line {
        Line::Row(r) => format!("Row {}", r + 1),
        Line::Col(c) => format!("Col {}", c + 1),
    }
}

/// Compact one-line rendering of a step: technique, actions, and a brief
/// reason. Enough detail to follow the solve and spot chain lengths without
/// reusing the CLI's (private) formatter.
fn format_step(step: &Step) -> String {
    let actions: Vec<String> = step
        .actions
        .iter()
        .map(|a| match a {
            Action::Place { row, col, value } => format!("{value}@{}", cell_label(*row, *col)),
            Action::Eliminate { row, col, value } => format!("-{value} {}", cell_label(*row, *col)),
        })
        .collect();

    let reason = match &step.reason {
        Reason::XyChainElimination {
            chain,
            eliminated_value,
        } => {
            let path: Vec<String> = chain.iter().map(|(r, c)| cell_label(*r, *c)).collect();
            format!(
                "XY-Chain len={} ({}) elim {eliminated_value}",
                chain.len(),
                path.join("->")
            )
        }
        Reason::SingleCandidate { row, col } => {
            format!("naked single {}", cell_label(*row, *col))
        }
        Reason::UniqueInLine { line, value } => {
            format!("hidden single {value} in {}", line_label(line))
        }
        Reason::SetInLine { line, values, .. } => {
            format!("naked set {values:?} in {}", line_label(line))
        }
        Reason::FishPattern { value, .. } => format!("fish on {value}"),
        Reason::PermutationElimination { line, .. } => {
            format!("permutation in {}", line_label(line))
        }
        Reason::DualCluePermutationElimination { line, .. } => {
            format!("dual-clue permutation in {}", line_label(line))
        }
        Reason::AlsXzElimination {
            als_a,
            als_b,
            rcc_value,
            eliminated_value,
        } => {
            let fmt_als = |als: &[(usize, usize)]| -> String {
                als.iter()
                    .map(|(r, c)| cell_label(*r, *c))
                    .collect::<Vec<_>>()
                    .join("+")
            };
            // Wing patterns are tiny ALS-XZ: |A|=|B|=1 is an XY-Wing, and
            // |A|+|B|=3 (one bivalue, one size-2 ALS) is an XYZ/W-Wing. Tag
            // the size so we can tell Hard-tier wings from genuine Expert ALS.
            let tag = match (als_a.len(), als_b.len()) {
                (1, 1) => " [XY-Wing-sized]",
                (1, 2) | (2, 1) => " [XYZ/W-Wing-sized]",
                _ => "",
            };
            format!(
                "ALS-XZ |A|={} |B|={} rcc={rcc_value} elim {eliminated_value} (A={}, B={}){tag}",
                als_a.len(),
                als_b.len(),
                fmt_als(als_a),
                fmt_als(als_b),
            )
        }
        Reason::ForcingChainElimination {
            assumed_cell,
            assumed_value,
        } => format!(
            "forcing chain {}={assumed_value}",
            cell_label(assumed_cell.0, assumed_cell.1)
        ),
        Reason::InitialClueConstraint { .. } => "initial clue pruning".to_string(),
        Reason::VisibilityForcing { line, .. } => {
            format!("visibility forcing in {}", line_label(line))
        }
    };

    format!("[{:?}] {}  ({reason})", step.technique, actions.join(", "))
}
