# skyscrapers-generator

Generates Skyscrapers puzzles with a guaranteed unique solution. Also exposes WebAssembly bindings consumed by [skyscrapers-player](../skyscrapers-player/README.md) and [skyscrapers-web](../skyscrapers-web/README.md).

## Pipeline

Generation runs in two stages:

1. **Stage A — Solution + full clues.** Sample an `n×n` Latin square with [`latin-sampler`](https://crates.io/crates/latin-sampler) (Jacobson–Matthews MCMC), convert it to a 1-based `Solution`, and derive all four edges of clues with `derive_clues`.
2. **Stage B — Greedy removal.** Remove board cells first, then clues, in randomized order. Each removal is kept only if the puzzle remains uniquely solvable (verified by a `Solver` implementation — `BacktrackingSolver` by default).

When `GeneratorParams::with_target_difficulty(...)` is set, Stage A/B are retried with fresh Latin squares until a puzzle matching the requested `Difficulty` is produced, or until `max_attempts` is exhausted (returns `GenerateError::MaxAttemptsExceeded`).

## API

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use skyscrapers_generator::{generate, GeneratorParams};

let mut rng = ChaCha20Rng::seed_from_u64(42);
let params = GeneratorParams::new(7);
let (puzzle, _solution) = generate(&mut rng, &params).expect("generation succeeds");
println!("{puzzle}");
```

Key items:

- `solution_from_latin_square(&LatinSquare) -> Solution` — converts a 0-based `LatinSquare` to a 1-based `Solution`.
- `derive_clues(&Solution) -> Clues` — thin convenience wrapper around `Clues::from_solution`.
- `GeneratorParams::new(n)` plus builders `with_solver`, `with_target_difficulty`, `with_max_attempts`.
- `generate(rng, &params) -> Result<(Puzzle, Solution), GenerateError>`.

## WebAssembly

The crate is built as both `rlib` and `cdylib`. On `wasm32` targets it pulls in `wasm-bindgen` + `serde-wasm-bindgen` to expose the generator and the logic solver to JavaScript.

```bash
wasm-pack build --target web skyscrapers-generator
```

This produces `skyscrapers-generator/pkg/`, which `skyscrapers-player`'s `WasmEngine` imports via a `file:` reference.
