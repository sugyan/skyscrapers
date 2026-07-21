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
use skyscrapers_solver::logic::bottleneck::compute_bottlenecks;
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
        #[arg(short, long, default_value_t = 7, value_parser = clap::value_parser!(u64).range(1..=9))]
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

    /// Reproduce a single puzzle (by seed/n/difficulty, same as the CLI
    /// `generate`) and print its "difficulty texture": how the required
    /// technique tiers are distributed across the solve. Beyond the headline
    /// tier, it reports how many *separate* forced stalls need the top tier
    /// (bursts), how long the longest stall is, and what tier the solver drops
    /// to after each — i.e. how "grindy vs. flowing" the puzzle is.
    Texture {
        /// Grid size (1-9)
        #[arg(short, long, default_value_t = 7, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// RNG seed (must match the seed used with the CLI `generate`)
        #[arg(long)]
        seed: u64,

        /// Target difficulty (must match what was used with `generate`)
        #[arg(long)]
        difficulty: Option<Difficulty>,
    },

    /// Scan a seed range at a fixed (n, target difficulty), compute each
    /// puzzle's difficulty texture, and rank within the tier — surfacing the
    /// "grindiest" and "smoothest" seeds so only a handful need hand-solving
    /// to check whether texture tracks felt difficulty.
    TextureScan {
        /// Grid size (1-9)
        #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// Target difficulty for the puzzles to scan
        #[arg(short, long)]
        difficulty: Difficulty,

        /// Number of seeds to test (0..samples)
        #[arg(short, long, default_value_t = 100)]
        samples: u64,

        /// Maximum generation attempts per seed
        #[arg(long, default_value_t = 500)]
        max_attempts: usize,

        /// How many extreme seeds to list at each end
        #[arg(long, default_value_t = 5)]
        top: usize,

        /// Emit the markdown subsection used in docs/logic-solver-analysis.md
        /// (distribution + grindiest/smoothest tables) instead of the plain
        /// text explorer output.
        #[arg(long)]
        markdown: bool,

        /// Always include this seed as a labelled reference row (e.g. a puzzle
        /// you played), even if it is outside `0..samples`. Markdown mode only.
        #[arg(long)]
        reference: Option<u64>,
    },

    /// Reproduce one or more puzzles (by seed/n/difficulty, same as the CLI
    /// `generate`) and report the *bottleneck-count* difficulty metric (Stage 2
    /// texture): how many times the solver must reach for the top tier before a
    /// cheaper cascade finishes the surrounding work. Exploratory — prints one
    /// row per seed so felt difficulty can be checked against the count.
    Bottleneck {
        /// Grid size (1-9)
        #[arg(short, long, default_value_t = 7, value_parser = clap::value_parser!(u64).range(1..=9))]
        n: u64,

        /// Target difficulty (must match what was used with `generate`)
        #[arg(short, long)]
        difficulty: Option<Difficulty>,

        /// One or more RNG seeds to analyse (must match seeds used with `generate`)
        #[arg(required = true)]
        seeds: Vec<u64>,
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
        Command::Texture {
            n,
            seed,
            difficulty,
        } => texture(n as usize, seed, difficulty),
        Command::TextureScan {
            n,
            difficulty,
            samples,
            max_attempts,
            top,
            markdown,
            reference,
        } => texture_scan(
            n as usize,
            difficulty,
            samples,
            max_attempts,
            top,
            markdown,
            reference,
        ),
        Command::Bottleneck {
            n,
            difficulty,
            seeds,
        } => bottleneck(n as usize, difficulty, seeds),
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
        let (puzzle, _sol, _diff) = match generate(&mut rng, &params) {
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
                "prefixpermutation" => Ok(Technique::PrefixPermutation),
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
        let (puzzle, _sol, _diff) = match generate(&mut rng, &params) {
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
const TECHNIQUE_ROWS: [Technique; 14] = [
    Technique::NakedSingles,
    Technique::CluePruning,
    Technique::HiddenSingles,
    Technique::PrefixPermutation,
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
    gen_failed: usize,
    tech_puzzles: BTreeMap<Technique, usize>,
    tech_steps: BTreeMap<Technique, usize>,
}

impl BatchTotals {
    /// Puzzles actually generated (solved into a tier or left unsolved).
    fn generated(&self) -> usize {
        self.counts.values().sum::<usize>() + self.unsolved
    }
}

fn report_batch(n: usize, samples: u64) -> BatchTotals {
    let mut totals = BatchTotals {
        counts: Default::default(),
        unsolved: 0,
        gen_failed: 0,
        tech_puzzles: Default::default(),
        tech_steps: Default::default(),
    };
    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n);
        let Ok((puzzle, _sol, _diff)) = generate(&mut rng, &params) else {
            totals.gen_failed += 1;
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
        let Ok((puzzle, _sol, _diff)) = generate(&mut rng, &params) else {
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
        "Each cell shows `used / harder / unsolvable` for puzzles generated at\nthe target difficulty and re-solved with the technique disabled\n(`max_attempts={necessity_attempts}`). Counts are over the seeds that\nsuccessfully generated a puzzle at the target — failed seeds are skipped,\nso the per-cell denominator is the matching Target Yield above (every\nseed, for the tiers shown here). `used` counts only techniques that\nsurface as top-level solve steps; a technique firing solely inside\nforcing-chain propagation is not counted (the `harder`/`unsolvable`\noutcomes are unaffected).\n"
    );
    let nec_sizes = [5usize, 6, 7];
    let nec_tiers = [Difficulty::Hard, Difficulty::Expert, Difficulty::Master];
    let nec_techs = [
        Technique::XyChain,
        Technique::AlsXz,
        Technique::DualCluePermutation,
    ];
    let mut nec_genfail = 0u64;
    for tech in nec_techs {
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
                    // Generation failures depend only on (n, tier), not on the
                    // disabled technique, so count them once (first tech pass)
                    // rather than once per section.
                    if tech == nec_techs[0] {
                        nec_genfail += samples - tested;
                    }
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
    let mut batch_genfail = 0usize;
    for (n, b) in &batches {
        batch_genfail += b.gen_failed;
        let tier_cells: Vec<String> = ALL_DIFFICULTIES
            .iter()
            .map(|d| b.counts.get(d).copied().unwrap_or(0).to_string())
            .collect();
        // Denominator is puzzles actually generated, so a generation failure
        // can't silently inflate the success rate.
        let generated = b.generated();
        let solved = generated - b.unsolved;
        let pct = if generated == 0 {
            0.0
        } else {
            solved as f64 / generated as f64 * 100.0
        };
        println!(
            "| {n} | {} | {} | {:.0}% |",
            tier_cells.join(" | "),
            b.unsolved,
            pct
        );
    }
    if batch_genfail > 0 {
        println!(
            "\n> Note: {batch_genfail} seed(s) failed generation and are excluded;\n> Success is over puzzles actually generated, not all {samples} seeds."
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

    report_texture_section(samples, necessity_attempts);
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
    let (puzzle, _sol, _diff) = match generate(&mut rng, &params) {
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

/// Difficulty-texture profile derived purely from a solve trace: how the
/// required technique tiers are spread across the solve. A "burst" is a
/// maximal run of consecutive same-tier steps; the number of bursts at the
/// top tier is the count of *separate* forced stalls that needed it.
///
/// All logic techniques are monotone eliminations, so the trace order is a
/// faithful "easiest-available-first" walk: a tier-`t` step only fires when
/// nothing cheaper could — which is what makes burst counting meaningful.
struct TextureProfile {
    /// Overall difficulty (max tier), `None` if unsolved by logic.
    difficulty: Option<Difficulty>,
    /// Total solving steps, excluding init-only CluePruning.
    total_steps: usize,
    /// Per tier: the length of each maximal same-tier run.
    tier_bursts: BTreeMap<Difficulty, Vec<usize>>,
    /// For each top-tier burst, the tier the solver dropped to immediately
    /// after (the "relief"). Bursts that ended the solve are counted in
    /// `solved_after` instead.
    relief_hist: BTreeMap<Difficulty, usize>,
    solved_after: usize,
}

impl TextureProfile {
    fn top_bursts(&self) -> &[usize] {
        self.difficulty
            .and_then(|d| self.tier_bursts.get(&d))
            .map_or(&[], Vec::as_slice)
    }
    /// Number of forced stalls at the top tier (= burst count).
    fn depth(&self) -> usize {
        self.top_bursts().len()
    }
    /// Total top-tier steps.
    fn top_steps(&self) -> usize {
        self.top_bursts().iter().sum()
    }
    /// Longest single top-tier stall.
    fn max_burst(&self) -> usize {
        self.top_bursts().iter().copied().max().unwrap_or(0)
    }
    /// Top-tier bursts whose relief was *not* all the way down to Easy — i.e.
    /// the solver only dropped to a mid tier before needing the top tier again.
    fn hard_reliefs(&self) -> usize {
        self.relief_hist
            .iter()
            .filter(|(t, _)| **t != Difficulty::Easy)
            .map(|(_, c)| *c)
            .sum()
    }
    /// Ranking key: grindier textures sort first (descending) — more forced
    /// stalls, then more top-tier work, then a longer single stall.
    fn grind_key(&self) -> (usize, usize, usize) {
        (self.depth(), self.top_steps(), self.max_burst())
    }
}

fn compute_texture(steps: &[Step], difficulty: Option<Difficulty>) -> TextureProfile {
    // CluePruning runs once at init and is not a "move you find" while
    // solving, so it is not part of the flow.
    let tiers: Vec<Difficulty> = steps
        .iter()
        .filter(|s| s.technique != Technique::CluePruning)
        .map(|s| s.technique.difficulty())
        .collect();

    // Collapse into maximal same-tier runs.
    let mut runs: Vec<(Difficulty, usize)> = Vec::new();
    for &t in &tiers {
        match runs.last_mut() {
            Some(last) if last.0 == t => last.1 += 1,
            _ => runs.push((t, 1)),
        }
    }

    let mut tier_bursts: BTreeMap<Difficulty, Vec<usize>> = Default::default();
    let mut relief_hist: BTreeMap<Difficulty, usize> = Default::default();
    let mut solved_after = 0usize;
    for (idx, &(tier, len)) in runs.iter().enumerate() {
        tier_bursts.entry(tier).or_default().push(len);
        if Some(tier) == difficulty {
            match runs.get(idx + 1) {
                Some(&(next, _)) => *relief_hist.entry(next).or_default() += 1,
                None => solved_after += 1,
            }
        }
    }

    TextureProfile {
        difficulty,
        total_steps: tiers.len(),
        tier_bursts,
        relief_hist,
        solved_after,
    }
}

fn texture(n: usize, seed: u64, difficulty: Option<Difficulty>) {
    // Reproduce the puzzle exactly as `skyscrapers-cli generate` would.
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut params = GeneratorParams::new(n);
    if let Some(d) = difficulty {
        params = params.with_target_difficulty(d);
    }
    let (puzzle, _sol, _diff) = match generate(&mut rng, &params) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: generation failed: {e}");
            std::process::exit(1);
        }
    };

    println!("{puzzle}");
    let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
    let prof = compute_texture(&result.steps, result.difficulty);

    println!("\n=== Difficulty texture (n={n}, seed={seed}) ===");
    let Some(d) = prof.difficulty else {
        println!("  overall difficulty: UNSOLVABLE (by logic)");
        return;
    };
    println!("  overall difficulty: {d}");
    println!("  solving steps (excl. CluePruning): {}", prof.total_steps);

    println!("\n  --- top tier: {d} ---");
    println!(
        "  forced stalls (bursts): {}   top-tier steps: {}   longest stall: {}",
        prof.depth(),
        prof.top_steps(),
        prof.max_burst()
    );
    println!("  burst sizes: {:?}", prof.top_bursts());
    let mut relief: Vec<String> = prof
        .relief_hist
        .iter()
        .map(|(t, c)| format!("{t} x{c}"))
        .collect();
    if prof.solved_after > 0 {
        relief.push(format!("solved x{}", prof.solved_after));
    }
    println!(
        "  relief after each stall: {}",
        if relief.is_empty() {
            "—".to_string()
        } else {
            relief.join(", ")
        }
    );

    println!("\n  --- all tiers ---");
    for (tier, sizes) in &prof.tier_bursts {
        let total: usize = sizes.iter().sum();
        // Pad a String, not the Display value directly — `Difficulty`'s
        // Display ignores fill/width, so `{tier:<7}` would not align.
        let tier = tier.to_string();
        println!(
            "  {tier:<7} bursts={:<3} steps={:<3} sizes={sizes:?}",
            sizes.len(),
            total,
        );
    }
}

/// Reproduce each seed's puzzle and print its bottleneck-count profile
/// (Stage 2 texture). One row per seed.
///
/// `bneck` = number of top-tier rounds (waves of hard reasoning needed to
/// break each stall) — the headline metric. `rel`/`bld` split those into
/// release rounds (`cascade > 0`, a placement was unlocked) vs buildup rounds
/// (`cascade == 0`, candidate-only grind) — texture that explains *why* a
/// puzzle is hard (staged vs grind-then-flow). The `rounds (width->cascade)`
/// column shows each round's available keys and the cells its cascade placed.
fn bottleneck(n: usize, difficulty: Option<Difficulty>, seeds: Vec<u64>) {
    println!("=== Bottleneck metric (n={n}) ===");
    println!(
        "  {:>6}  {:<7}  {:>5}  {:>3}  {:>3}  {:>9}  rounds (width->cascade)",
        "seed", "tier", "bneck", "rel", "bld", "top_steps"
    );
    for seed in seeds {
        // Reproduce the puzzle exactly as `skyscrapers-cli generate` would.
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut params = GeneratorParams::new(n);
        if let Some(d) = difficulty {
            params = params.with_target_difficulty(d);
        }
        let puzzle = match generate(&mut rng, &params) {
            Ok((puzzle, _sol, _diff)) => puzzle,
            Err(e) => {
                println!("  {seed:>6}  generation failed: {e}");
                continue;
            }
        };

        let prof = compute_bottlenecks(&puzzle);
        let Some(tier) = prof.top_tier else {
            println!("  {seed:>6}  UNSOLVABLE (by logic)");
            continue;
        };
        // `Difficulty`'s Display ignores fill/width, so stringify before padding.
        let tier = tier.to_string();
        let rounds: Vec<String> = prof
            .rounds
            .iter()
            .map(|r| format!("{}->{}", r.width, r.cascade))
            .collect();
        let unsolved = if prof.solved { "" } else { "  [UNSOLVED]" };
        println!(
            "  {seed:>6}  {tier:<7}  {:>5}  {:>3}  {:>3}  {:>9}  [{}]{unsolved}",
            prof.bottlenecks(),
            prof.releases(),
            prof.buildup_rounds(),
            prof.total_top_steps(),
            rounds.join(", "),
        );
    }
}

/// The n=5 hard puzzle whose grindy middle first motivated this metric.
const TEXTURE_REFERENCE_SEED: u64 = 20260702;

/// Intro prose for the generated "Difficulty Texture" doc section. Kept in
/// code so `report` and the committed doc stay in sync.
const TEXTURE_INTRO: &str = "\"Difficulty texture\" looks past the headline tier at how the \
top-tier work is spread across the solve. Grouping the trace into maximal same-tier runs \
(\"bursts\", excluding init-only CluePruning), we report, for the top tier: **stalls** = the \
number of *separate* forced stalls that needed it, **topSteps** = total top-tier steps, and \
**longest stall** = the largest single burst. Many/long stalls with little relief feel grindier \
than a single hard move that unlocks an easy cascade — so two puzzles at the same tier can differ \
widely here. Ranking is **stalls-first** (then topSteps, then longest stall); note this puts a \
single long unbroken burst (1 stall but high topSteps) at the \"smooth\" end even though it is a \
long slog — how to weight stall *count* against burst *length* is an open question these numbers \
are meant to help settle. Use `explain` / `texture` (see Reproduction) to inspect a listed seed.";

/// Generate `0..samples` puzzles at `(n, difficulty)`, solve each, and keep
/// the texture profiles of those that landed on the target tier. Returns the
/// rows plus generation-failure / off-target counts.
fn collect_texture(
    n: usize,
    difficulty: Difficulty,
    samples: u64,
    max_attempts: usize,
) -> (Vec<(u64, TextureProfile)>, u64, u64) {
    let mut rows = Vec::new();
    let (mut gen_failed, mut off_target) = (0u64, 0u64);
    for seed in 0..samples {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let params = GeneratorParams::new(n)
            .with_target_difficulty(difficulty)
            .with_max_attempts(max_attempts);
        let Ok((puzzle, _sol, _diff)) = generate(&mut rng, &params) else {
            gen_failed += 1;
            continue;
        };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        let prof = compute_texture(&result.steps, result.difficulty);
        // Only compare texture within the same achieved tier.
        if prof.difficulty == Some(difficulty) {
            rows.push((seed, prof));
        } else {
            off_target += 1;
        }
    }
    (rows, gen_failed, off_target)
}

/// Texture profile of a single reproduced puzzle (same seed scheme as
/// `skyscrapers-cli generate`). `None` if generation fails.
fn texture_of(n: usize, seed: u64, difficulty: Option<Difficulty>) -> Option<TextureProfile> {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut params = GeneratorParams::new(n);
    if let Some(d) = difficulty {
        params = params.with_target_difficulty(d);
    }
    let (puzzle, _sol, _diff) = generate(&mut rng, &params).ok()?;
    let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
    Some(compute_texture(&result.steps, result.difficulty))
}

/// `(min, median, max)` of a list (0s if empty).
fn stats(mut v: Vec<usize>) -> (usize, usize, usize) {
    v.sort_unstable();
    (
        v.first().copied().unwrap_or(0),
        v.get(v.len() / 2).copied().unwrap_or(0),
        v.last().copied().unwrap_or(0),
    )
}

fn texture_scan(
    n: usize,
    difficulty: Difficulty,
    samples: u64,
    max_attempts: usize,
    top: usize,
    markdown: bool,
    reference: Option<u64>,
) {
    let (rows, gen_failed, off_target) = collect_texture(n, difficulty, samples, max_attempts);

    if markdown {
        let ref_prof = reference.and_then(|s| texture_of(n, s, Some(difficulty)));
        format_texture_tier(difficulty, &rows, reference.zip(ref_prof.as_ref()));
        return;
    }

    println!("=== Texture scan (n={n}, target={difficulty}, samples={samples}) ===");
    println!(
        "  on-target: {}   gen_failed: {gen_failed}   off_target: {off_target}\n",
        rows.len()
    );
    // Raw per-seed columns (already in seed order) — left deliberately
    // un-fused so the score weighting can be tuned outside this tool.
    println!("  seed | steps | stalls | topSteps | maxStall | midRelief");
    println!("  -----+-------+--------+----------+----------+----------");
    for (seed, p) in &rows {
        println!(
            "  {seed:>4} | {:>5} | {:>6} | {:>8} | {:>8} | {:>9}",
            p.total_steps,
            p.depth(),
            p.top_steps(),
            p.max_burst(),
            p.hard_reliefs(),
        );
    }

    if rows.is_empty() {
        return;
    }
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| rows[b].1.grind_key().cmp(&rows[a].1.grind_key()));
    let k = top.min(order.len());
    let line = |i: usize| {
        let (seed, p) = &rows[i];
        format!(
            "    seed={seed}  stalls={} topSteps={} maxStall={}",
            p.depth(),
            p.top_steps(),
            p.max_burst()
        )
    };
    println!("\n  grindiest {k} (hand-solve these — expect 'harder within {difficulty}'):");
    for &i in order.iter().take(k) {
        println!("{}", line(i));
    }
    println!("\n  smoothest {k} (expect 'flows'):");
    for &i in order.iter().rev().take(k) {
        println!("{}", line(i));
    }
}

/// One tier's markdown subsection for the doc: a distribution line, then the
/// grindiest / smoothest tables, then an optional labelled reference row.
fn format_texture_tier(
    difficulty: Difficulty,
    rows: &[(u64, TextureProfile)],
    reference: Option<(u64, &TextureProfile)>,
) {
    println!("### {difficulty}\n");
    if rows.is_empty() {
        println!("_No puzzles generated at this tier in the sampled seeds._\n");
        return;
    }
    let (smin, smed, smax) = stats(rows.iter().map(|(_, p)| p.depth()).collect());
    let (_, tmed, _) = stats(rows.iter().map(|(_, p)| p.top_steps()).collect());
    let (_, bmed, _) = stats(rows.iter().map(|(_, p)| p.max_burst()).collect());
    let mut hist: BTreeMap<usize, usize> = Default::default();
    for (_, p) in rows {
        *hist.entry(p.depth()).or_default() += 1;
    }
    let hist_str: Vec<String> = hist.iter().map(|(k, v)| format!("{k}:{v}")).collect();
    println!(
        "{} puzzles. Forced stalls min/median/max = {smin}/{smed}/{smax} \
         (stalls:count → {}); median topSteps {tmed}, median longest stall {bmed}.\n",
        rows.len(),
        hist_str.join(", "),
    );

    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| rows[b].1.grind_key().cmp(&rows[a].1.grind_key()));
    let k = 10.min(order.len());
    let table = |title: &str, idxs: &[usize]| {
        println!("{title}\n");
        println!("| seed | stalls | topSteps | longest stall |");
        println!("|------|--------|----------|---------------|");
        for &i in idxs {
            let (s, p) = &rows[i];
            println!(
                "| {s} | {} | {} | {} |",
                p.depth(),
                p.top_steps(),
                p.max_burst()
            );
        }
        println!();
    };
    let grind: Vec<usize> = order.iter().take(k).copied().collect();
    let smooth: Vec<usize> = order.iter().rev().take(k).copied().collect();
    table(
        &format!("**Grindiest {k}** (expect these to feel hardest):"),
        &grind,
    );
    table(
        &format!("**Smoothest {k}** (expect these to flow):"),
        &smooth,
    );

    if let Some((seed, p)) = reference {
        println!(
            "_Reference — the puzzle that first motivated this metric_: `seed={seed}` → \
             {} stalls, {} topSteps, longest stall {}.\n",
            p.depth(),
            p.top_steps(),
            p.max_burst(),
        );
    }
}

/// Difficulty-texture section for `docs/logic-solver-analysis.md`, emitted as
/// part of `report`. n=5 only (the size with hand-solving intuition), for the
/// two tiers where texture actually varies.
fn report_texture_section(samples: u64, max_attempts: usize) {
    let n = 5usize;
    println!("\n## Difficulty Texture (n={n}, {samples} seeds per tier)\n");
    println!("{TEXTURE_INTRO}\n");
    for difficulty in [Difficulty::Hard, Difficulty::Expert] {
        eprintln!("texture n={n} {difficulty}…");
        let (rows, _gen_failed, _off_target) =
            collect_texture(n, difficulty, samples, max_attempts);
        let ref_prof = if difficulty == Difficulty::Hard {
            texture_of(n, TEXTURE_REFERENCE_SEED, Some(Difficulty::Hard))
        } else {
            None
        };
        format_texture_tier(
            difficulty,
            &rows,
            ref_prof.as_ref().map(|p| (TEXTURE_REFERENCE_SEED, p)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// compute_texture only reads `technique`, so a dummy reason/actions is fine.
    fn step(t: Technique) -> Step {
        Step {
            technique: t,
            actions: Vec::new(),
            reason: Reason::SingleCandidate { row: 0, col: 0 },
        }
    }

    #[test]
    fn bursts_relief_and_clue_pruning_excluded() {
        use Technique::*;
        // CluePruning is dropped; then 2 easy, a 3-long hard stall, relief to
        // easy, then a 1-long hard stall that ends the solve.
        let steps: Vec<Step> = [
            CluePruning,
            NakedSingles,
            HiddenSingles,
            NakedSets,
            XWing,
            SimplePermutation,
            HiddenSingles,
            NakedSets,
        ]
        .into_iter()
        .map(step)
        .collect();

        let p = compute_texture(&steps, Some(Difficulty::Hard));
        assert_eq!(p.total_steps, 7, "CluePruning excluded from the flow");
        assert_eq!(p.top_bursts(), &[3, 1]);
        assert_eq!(p.depth(), 2);
        assert_eq!(p.top_steps(), 4);
        assert_eq!(p.max_burst(), 3);
        // First hard stall relieved to Easy; the second ended the solve.
        assert_eq!(p.relief_hist.get(&Difficulty::Easy).copied(), Some(1));
        assert_eq!(p.solved_after, 1);
        assert_eq!(
            p.tier_bursts.get(&Difficulty::Easy).map(Vec::as_slice),
            Some([2usize, 1].as_slice()),
        );
    }

    #[test]
    fn unsolved_has_no_top_tier() {
        let steps: Vec<Step> = [Technique::NakedSingles, Technique::HiddenSingles]
            .into_iter()
            .map(step)
            .collect();
        let p = compute_texture(&steps, None);
        assert_eq!(p.depth(), 0);
        assert!(p.top_bursts().is_empty());
        assert_eq!(p.solved_after, 0);
    }
}
