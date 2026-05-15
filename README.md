# skyscrapers

A Rust workspace for generating and solving [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles, plus a React player that runs the solver in the browser via WebAssembly.

**Demo:** <https://sugyan.com/skyscrapers/>

## What is Skyscrapers?

Skyscrapers is played on an n×n grid. Each row and column is a permutation of `1..=n` (a Latin square). Clue numbers along the edges indicate how many "buildings" are visible when looking down that row or column from that direction — taller buildings hide shorter ones behind them. The goal is to fill the grid so that every clue is satisfied.

This project targets `n = 7` and `n = 8` as the primary sizes for human play, while supporting `n = 1..=9` throughout.

## Workspace layout

| Crate / package | Description |
|---|---|
| [skyscrapers-core](skyscrapers-core/README.md) | Shared domain types (`Solution`, `Board`, `Clues`, `Puzzle`) and clue derivation. |
| [skyscrapers-solver](skyscrapers-solver/README.md) | Backtracking solver for uniqueness checking and a logic solver with human-traceable techniques for difficulty rating. |
| [skyscrapers-generator](skyscrapers-generator/README.md) | Two-stage puzzle generator (Latin-square sampling + greedy clue/cell removal) with optional WebAssembly bindings. |
| [skyscrapers-cli](skyscrapers-cli/README.md) | `skyscrapers` binary exposing `generate` and `solve` subcommands. |
| [skyscrapers-analysis](skyscrapers-analysis/README.md) | Dev-only workspace analyses (target-yield, technique necessity, batch difficulty). Not published. |
| [skyscrapers-player](skyscrapers-player/README.md) | React component plus a pluggable `SkyscrapersEngine` interface; bundled `WasmEngine` runs the solver in-process. |
| [skyscrapers-web](skyscrapers-web/README.md) | Demo web app wiring `WasmEngine` + a generation form around `<Player>`. |

Dependency flow is always `core → solver → generator → cli`, with no cycles.

## Quick start: CLI

```bash
# Generate a puzzle (default n=7, random seed printed to stderr)
cargo run -p skyscrapers-cli -- generate -n 7 --seed 42

# Pipe a generated puzzle into the solver
cargo run -p skyscrapers-cli -- generate -n 5 --seed 42 \
  | cargo run -p skyscrapers-cli -- solve
```

See [skyscrapers-cli/README.md](skyscrapers-cli/README.md) for the full list of flags (including `--difficulty` for the generator and `--logic` for the step-by-step trace).

## Quick start: web demo (local)

The web app consumes `skyscrapers-generator` via `wasm-pack`, so the WASM artifact must be built first.

```bash
# 1. Build the WASM package
wasm-pack build --target web skyscrapers-generator

# 2. Start the dev server
cd skyscrapers-web
npm ci
npm run dev
```

The hosted version of the same app is at <https://sugyan.com/skyscrapers/>.

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --check
```

The solver crate also ships a benchmark suite that uses the nightly `#![feature(test)]` harness: `cargo +nightly bench -p skyscrapers-solver`.

## License

MIT. See [LICENSE](LICENSE).
