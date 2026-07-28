# CLAUDE.md — Skyscrapers Puzzle Generator

## Project Overview

An application that automatically generates [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles with guaranteed unique solutions.

A Skyscrapers puzzle is played on an n×n grid where each row and column is a permutation of 1..=n (a Latin square). Clue numbers on the edges indicate how many "buildings" are visible when looking along that row/column from that direction — taller buildings hide shorter ones behind them.

### Goals

- Generate Skyscrapers puzzles for n=7–8
- Guarantee unique solutions via solver-backed validation
- Rate difficulty with a logic-only (human-technique) solver

All three are implemented; see "Logic Solver + Difficulty" below.

## Architecture

### Workspace Structure

```
skyscrapers/
├── Cargo.toml                (workspace root)
├── docs/                     logic-solver-analysis.md (generated + hand-written)
├── skyscrapers-core/         Shared types + clue derivation
├── skyscrapers-solver/       Uniqueness verifier (backtracking) + logic solver
│                             with human techniques and difficulty rating
├── skyscrapers-generator/    Puzzle generator (also exposes WASM bindings)
├── skyscrapers-cli/          CLI binary (generate + solve)
├── skyscrapers-analysis/     Dev-only analysis/benchmarking tools (not shipped)
├── skyscrapers-player/       React components + engine interface (npm pkg, not published)
├── skyscrapers-web/          Demo web app — thin shell around skyscrapers-player
└── skyscrapers-tauri/        Desktop app (Tauri v2) — same shell, native engine
```

There is no separate `skyscrapers-logic` crate: the logic solver lives in
`skyscrapers-solver/src/logic/`, since it shares the candidate representation
with the backtracking solver.

### Dependency Graph

```
skyscrapers-core            ← all other crates depend on this
skyscrapers-solver          ← depends on core
skyscrapers-generator       ← depends on core, solver, latin-sampler
skyscrapers-cli             ← depends on core, solver, generator, clap
skyscrapers-analysis        ← depends on core, solver (analysis-hooks), generator, clap
skyscrapers-tauri/src-tauri ← depends on core, solver, generator, tauri
```

No circular dependencies. Flow is always: core → solver → generator → cli.
`skyscrapers-analysis` is a development crate for running workspace-wide
analyses (e.g. regenerating `docs/logic-solver-analysis.md`); it is
`publish = false` and not part of the end-user surface.
`skyscrapers-tauri/src-tauri` is excluded from the workspace's
`default-members` because it pulls in platform webview dependencies.

### External Dependencies

- [`latin-sampler`](https://crates.io/crates/latin-sampler) — Latin square generation via Jacobson-Matthews MCMC
- `rand`, `rand_chacha` — seedable RNG
- `clap` — CLI argument parsing (derive mode)

## Core Types (skyscrapers-core)

- **`Solution`** — A complete n×n grid (1-based values). `new(n, cells)`, `n()`, `get(r, c)`, `cells()`, `Display`
- **`Board`** — An n×n grid with optional cells. `new_empty(n)`, `get(r, c)`, `set(r, c, v)`
- **`Clues`** — Clue numbers for all 4 directions. `new_all_none(n)`, `from_solution(sol)`, per-direction accessors/setters
- **`Puzzle`** — `Board` + `Clues`. `Display` (box format), `FromStr` (parses box format)
- **`ParseError`** / **`SolutionParseError`** — Error types for `Puzzle::from_str` / `Solution::from_str`

Enabling the crate's `serde` feature derives `Serialize`/`Deserialize` on these
types; that is how the WASM bindings and the Tauri commands cross the boundary.

### Clue Derivation

`Clues::from_solution()` computes the visible building count for each direction. A building of height h is visible if no taller building appears before it from the viewing direction.

## Generator Pipeline

The generator has two stages:

1. **Stage A:** Generate a solution via `latin-sampler`, convert to `Solution`, derive full board + all clues
2. **Stage B:** Greedy removal of board cells and clues while preserving uniqueness. Board cells are removed first, then clues (two-phase strategy; may be changed to mixed strategy in the future)

### API (skyscrapers-generator)

- `solution_from_latin_square(ls) -> Solution` — converts 0-based LatinSquare to 1-based Solution
- `derive_clues(solution) -> Clues` — computes all clue numbers from a solution
- `generate(rng, params) -> Result<(Puzzle, Solution, Option<Difficulty>), GenerateError>` — end-to-end puzzle generation (Stage A + B). The third element is the difficulty the logic solver rated the generated puzzle at (`None` when the puzzle is harder than the logic solver can rate, only possible without a target difficulty)
- `GeneratorParams` — configuration: `n`, `solver`, `sampler_params`, `target_difficulty`, `max_attempts`. Built with `GeneratorParams::new(n)` (defaults: `BacktrackingSolver`, default sampler params, no target difficulty, 100 attempts) plus the builder methods `with_solver`, `with_target_difficulty`, `with_max_attempts`

When `target_difficulty` is set, `generate` retries up to `max_attempts` times
until the logic solver rates a puzzle at exactly that tier, and returns
`GenerateError` if the budget runs out.

## Logic Solver + Difficulty (skyscrapers-solver)

`LogicSolver` solves a puzzle using only human-traceable techniques and reports
the hardest tier it needed. It backs both the difficulty rating on generated
puzzles and the player's hint button.

- `solve_with_difficulty(puzzle, limit) -> SolveResult` — full solve with a step trace
- `next_step(puzzle, board)` / `next_step_with_candidates(puzzle, board, user_candidates)` — a single next deduction, used for hints

`Difficulty` has five tiers: `Easy`, `Medium`, `Hard`, `Expert`, `Master`. A
puzzle's tier is the hardest technique its solve required, so adding a *weaker*
technique can *lower* a puzzle's rating by attributing a deduction to a cheaper
tier — that is precisely what `PrefixPermutation` was added for.

The 14 techniques live in `skyscrapers-solver/src/logic/techniques/`:

| Tier | Techniques (`Technique::difficulty()` in `logic/difficulty.rs`) |
|------|------------|
| Easy | `NakedSingles`, `HiddenSingles` |
| Medium | `CluePruning`, `VisibilityAnalysis` |
| Hard | `NakedSets`, `XWing`, `XyChain`, `PrefixPermutation`, `SimplePermutation` |
| Expert | `AlsXz`, `PermutationEnumeration`, `DualCluePermutation` |
| Master | `SimpleForcingChain`, `FullForcingChain` |

`CluePruning` is the one exception to "tier = rating": it runs unconditionally at
init from the puzzle's starting clues, so a puzzle it fires on can still rate
`Easy`.

`docs/logic-solver-analysis.md` records how these behave in practice — per-size
difficulty distributions, target-difficulty yield, which techniques are
load-bearing, and two exploratory within-tier "texture" metrics. Its numeric
tables are regenerated by `skyscrapers-analysis`; the prose around them is
maintained by hand. **Read that document before changing technique dispatch
order or difficulty tiers**, and regenerate it afterwards.

`skyscrapers-solver` also exposes an `analysis-hooks` feature: an internal
switch that lets `skyscrapers-analysis` disable individual techniques to measure
their necessity. Not part of the end-user surface.

## Web / Player (npm packages)

- **`skyscrapers-player`** — React 19 components + the `SkyscrapersEngine` interface. The package is **transport-neutral: it ships no engine implementation**; each host injects its own. It exports `<Player>` (just the board, number pad, controls and hint panel) and `<PuzzleApp>` (the whole app — generation form + `<Player>` + seed footer + How to Play). Not published to npm. Two install paths: the monorepo uses `file:../skyscrapers-player`, and external projects install from the `player-dist` Git branch (`npm install github:sugyan/skyscrapers#player-dist`), which is rebuilt on every push to `main` by `.github/workflows/player-dist.yml`.
- **`skyscrapers-web`** — Demo application. Owns `WasmEngine` (runs the Rust solver in-process via WebAssembly) and keeps the current puzzle's parameters in the URL query string, then renders `<PuzzleApp>`. Tailwind v4 styling lives in the player; the web app just imports `skyscrapers-player/styles.css`.
- **`skyscrapers-tauri`** — Desktop app (Tauri v2). Owns `TauriEngine`, which calls the same Rust crates natively over IPC instead of WebAssembly, and renders the same `<PuzzleApp>` with no URL handling.

Anything both hosts need belongs in `<PuzzleApp>`; platform-specific behavior
stays in the host and is threaded through its `initialRequest` / `onGenerated` /
`onCleared` callbacks, so the player never learns about URLs or `window.history`.
The two apps drifted apart when they each had their own copy of this shell, so
resist reintroducing host-local UI.

Install + check (run in each package as needed):

```bash
# In skyscrapers-player/, skyscrapers-web/, or skyscrapers-tauri/
npm ci
npm run lint
npm run format:check
npm run typecheck   # player + tauri (the web app typechecks via `npm run build`)
npm test            # player + web
```

All three packages are linted and format-checked in CI: `web.yml` covers the
player and web app, `tauri-check.yml` covers the Tauri frontend before its
(much slower) Rust build.

The web build additionally depends on the WASM artifact produced by `wasm-pack build --target web skyscrapers-generator` — CI builds this before `npm ci` so the `file:../skyscrapers-generator/pkg` dependency resolves.

## Development

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

These are the same checks `.github/workflows/rust.yml` enforces. They target the
workspace's `default-members`, which excludes `skyscrapers-tauri/src-tauri`
(platform webview dependencies); `tauri-check.yml` builds that crate on three
OSes instead.

`skyscrapers-solver` ships a benchmark suite behind the `nightly-bench` feature,
because it uses the unstable `#![feature(test)]` harness. The feature gate is what
keeps stable `--all-targets` builds working:

```bash
cargo +nightly bench -p skyscrapers-solver --features nightly-bench
```

## CLI Usage

`skyscrapers-cli` provides a `skyscrapers` binary with two subcommands:

```bash
# Generate a puzzle (default n=7, random seed printed to stderr)
skyscrapers generate [-n <SIZE>] [--seed <SEED>] [--difficulty <LEVEL>]

# Solve a puzzle from file or stdin
skyscrapers solve [FILE] [--logic]

# Pipe: generate and immediately solve
skyscrapers generate -n 5 --seed 42 | skyscrapers solve
```

`--difficulty` takes `easy|medium|hard|expert|master` and makes the generator
retry until the logic solver rates the puzzle at exactly that tier. `--logic`
solves with the logic solver instead of backtracking and prints the
step-by-step reasoning trace.

## Conventions

- **n ≤ 9**: Project-wide constraint. `Solution::new`, `Board::new_empty`, `Clues::new_all_none` all assert `1..=9`. The text format assumes single-digit values.
- Cell values are **1-based** (1..=n) throughout the Skyscrapers domain types
- `latin-sampler` uses 0-based symbols; conversion happens at the boundary in `solution_from_latin_square`
- 2D storage: `Solution` and `Board` use `Vec<Vec<..>>`, accessed via `cells[r][c]`
- Rust edition 2024, MSRV 1.85
