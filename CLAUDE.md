# CLAUDE.md — Skyscrapers Puzzle Generator

## Project Overview

An application that automatically generates [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles with guaranteed unique solutions.

A Skyscrapers puzzle is played on an n×n grid where each row and column is a permutation of 1..=n (a Latin square). Clue numbers on the edges indicate how many "buildings" are visible when looking along that row/column from that direction — taller buildings hide shorter ones behind them.

### Goals

- Generate Skyscrapers puzzles for n=7–8
- Guarantee unique solutions via solver-backed validation
- (Future) Difficulty rating via logic-only solver

## Architecture

### Workspace Structure

```
skyscrapers/
├── Cargo.toml                (workspace root)
├── skyscrapers-core/         Shared types + clue derivation
├── skyscrapers-solver/       Uniqueness verifier (backtracking; SAT planned)
├── skyscrapers-generator/    Puzzle generator
├── skyscrapers-logic/        Logic solver + difficulty rating           [planned]
└── skyscrapers-cli/          CLI binary (generate + solve)              [planned]
```

### Dependency Graph

```
skyscrapers-core         ← all other crates depend on this
skyscrapers-solver       ← depends on core
skyscrapers-generator    ← depends on core, solver, latin-sampler
skyscrapers-logic        ← depends on core (future)
skyscrapers-cli          ← depends on core, solver, generator (future)
```

No circular dependencies. Flow is always: core → solver → generator → cli.

### External Dependencies

- [`latin-sampler`](https://crates.io/crates/latin-sampler) — Latin square generation via Jacobson-Matthews MCMC
- `rand`, `rand_chacha` — seedable RNG
- `varisat` — SAT solver for uniqueness verification (planned)

## Core Types (skyscrapers-core)

- **`Solution`** — A complete n×n grid (1-based values). `new(n, cells)`, `n()`, `get(r, c)`, `cells()`
- **`Board`** — An n×n grid with optional cells. `new_empty(n)`, `get(r, c)`, `set(r, c, v)`
- **`Clues`** — Clue numbers for all 4 directions. `new_all_none(n)`, `from_solution(sol)`, per-direction accessors/setters
- **`Puzzle`** — `Board` + `Clues`

### Clue Derivation

`Clues::from_solution()` computes the visible building count for each direction. A building of height h is visible if no taller building appears before it from the viewing direction.

## Generator Pipeline

The generator has two stages:

1. **Stage A:** Generate a solution via `latin-sampler`, convert to `Solution`, derive full board + all clues
2. **Stage B:** Greedy removal of board cells and clues while preserving uniqueness. Board cells are removed first, then clues (two-phase strategy; may be changed to mixed strategy in the future)

### API (skyscrapers-generator)

- `solution_from_latin_square(ls) -> Solution` — converts 0-based LatinSquare to 1-based Solution
- `derive_clues(solution) -> Clues` — computes all clue numbers from a solution
- `generate(rng, params) -> Puzzle` — end-to-end puzzle generation (Stage A + B)
- `GeneratorParams` — configuration: `n`, `solver`, `sampler_params`

## Implementation Status

### Phase 1: Foundation + Solver

| Step | Status |
|------|--------|
| Remove `latin-sampler/` from repo, use as external dep | Done |
| Workspace restructuring | Done |
| `skyscrapers-core` (types + clue derivation) | Done |
| `skyscrapers-generator` stage A (solution + clues) | Done |
| `skyscrapers-solver` (backtracking) | Done |
| `skyscrapers-solver` (SAT) | Not started |
| Solver benchmarks (n=7, 8) | Not started |

### Phase 2: Puzzle Generation

| Step | Status |
|------|--------|
| `skyscrapers-generator` stage B (greedy removal) | Done |
| Quality validation (uniqueness + clue count stats) | Not started |

### Phase 3: Logic Solver + Difficulty

| Step | Status |
|------|--------|
| `skyscrapers-logic` (human-technique solver) | Not started |
| Difficulty scoring | Not started |

### Phase 4: CLI

| Step | Status |
|------|--------|
| `skyscrapers-cli` crate setup | Not started |
| `generate` subcommand (options: size, seed, output format) | Not started |
| `solve` subcommand (read puzzle from stdin/file, print solution) | Not started |
| `Display` impl for `Puzzle` and `Solution` in core | Not started |

## Development

```bash
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
```

## CLI Design (planned)

`skyscrapers-cli` will be a single binary with two subcommands:

- **`generate`** — Generate a puzzle. Options: grid size (`-n`), RNG seed (`--seed`), output format.
- **`solve`** — Read a puzzle from stdin or file and print the solution.

Both subcommands require `Display` implementations for `Puzzle` and `Solution` in `skyscrapers-core` to produce human-readable text output.

## Known Issues / TODO

- **No range validation in `Board::set` / `Clues::set_*`**: Invalid values such as `Board::set(r, c, Some(0))` or `Clues::set_top(i, Some(99))` are silently accepted. `Solution::new` validates the value range, but `Board` and `Clues` do not. Validation should be added at the core boundary (e.g., `Board::set` should verify `v` is in `1..=n`; `Clues::set_*` should verify `v` is in `1..=n`).

## Conventions

- Cell values are **1-based** (1..=n) throughout the Skyscrapers domain types
- `latin-sampler` uses 0-based symbols; conversion happens at the boundary in `solution_from_latin_square`
- Row-major storage: index = r * n + c
- Rust edition 2024, MSRV 1.85
