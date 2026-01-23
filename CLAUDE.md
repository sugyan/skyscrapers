# CLAUDE.md — latin-sampler

## 0. Purpose

`latin-sampler` is a Rust library that generates (approximately) uniformly distributed **Latin squares** of order `n` (primary target: `n = 7` or `8`) using an MCMC sampler with **seedable RNG injection** for full reproducibility.

### Non-goals
- Proving mixing time or providing a mathematically rigorous uniformity guarantee.
- Solving / enumerating all Latin squares.
- Supporting very large `n` efficiently.

### Design constraints
- Deterministic output given the same seed and parameters.
- Always maintain the Latin property (no rejection / no invalid intermediate state).
- Simple, testable, auditable core move set.
- Reasonable performance for `n <= 10`, especially `7/8`.

### v0.1 scope (minimal)
- `LatinSquare` struct with basic operations
- `SamplerParams` and `sample()` function
- Essential unit tests only
- Deferred to v0.2+: `Sampler<R>` iterator, `serde` feature, `proptest`

## 1. Definitions

A Latin square of order `n` is an `n x n` array with symbols `{0..n-1}` such that:
- each row is a permutation of `{0..n-1}`
- each column is a permutation of `{0..n-1}`

We treat the sampler distribution as "approximately uniform" after sufficient burn-in and mixing.

## 2. Public API (v0.1 target)

### 2.1 Types

- `pub struct LatinSquare { n: usize, cells: Vec<u8> }`

Constraints:
- `n <= 255` (symbols stored as `u8`)
- `cells.len() == n*n`

Indexing:
- linear index `idx = r*n + c`

### 2.2 Constructors / accessors

- `impl LatinSquare`
  - `pub fn new_cyclic(n: usize) -> Self`
    - returns the cyclic Latin square: `L[r][c] = (r + c) mod n`
  - `pub fn n(&self) -> usize`
  - `pub fn get(&self, r: usize, c: usize) -> u8`
  - `pub(crate) fn set_unchecked(&mut self, r: usize, c: usize, v: u8)`

Note: `is_latin` is NOT part of the public API. The Latin property is an invariant enforced by construction and moves. A test-only helper `is_latin()` exists for validation.

### 2.3 Sampler parameters

- `pub struct SamplerParams`
  - `pub burn_in: u64`  // number of steps discarded
  - `pub steps: u64`    // number of steps after burn-in before returning
  - `pub thinning: u64` // optional; if >1, only return every k steps in iterator mode
  - `pub p_row_move: f64` // in [0,1]
  - `pub p_do_nothing: f64` // in [0,1], for aperiodicity

Provide:
- `impl Default for SamplerParams` with safe defaults for `n=7/8`:
  - `burn_in = 300_000`
  - `steps = 80_000`
  - `thinning = 1`
  - `p_row_move = 0.5`
  - `p_do_nothing = 0.01`

### 2.4 Sampling entry point (seedable via RNG injection)

- `pub fn sample<R: rand::Rng + ?Sized>(n: usize, rng: &mut R, params: &SamplerParams) -> LatinSquare`

RNG guidance (docs/examples):
- Recommend `rand_chacha::ChaCha20Rng` for reproducibility:
  - `let mut rng = ChaCha20Rng::from_seed([0u8; 32]);`

#### v0.2+: Sampler iterator
- `pub struct Sampler<R>` implementing `Iterator<Item = LatinSquare>`
  - created by `Sampler::new(n, rng, params)`
  - yields successive samples separated by `params.thinning`

## 3. Core algorithm

### 3.1 Overview

Use an MCMC random walk over the space of valid Latin squares using **always-valid** local moves:
- `row_cycle_move` (2-row cycle swap trade)
- `col_cycle_move` (2-column cycle swap trade)
plus occasional no-op to ensure aperiodicity.

Each step:
- With probability `p_do_nothing`: do nothing
- Else choose move type:
  - with probability `p_row_move`: apply `row_cycle_move`
  - else: apply `col_cycle_move`

### 3.2 Row-cycle move (always preserves Latin property)

Given square `L`:

1) Choose distinct rows `r1 != r2` uniformly.

2) Build inverse map for row `r1`:
- `pos[s] = c` where `L[r1][c] == s` for all `s`.

3) Define a permutation on symbols:
- `perm[s] = L[r2][pos[s]]`
This is a permutation of `{0..n-1}`.

4) Select a random non-trivial cycle (length >= 2) from `perm`.
- If only fixed points are encountered after a small number of retries, treat as no-op.

5) For each symbol `s` in the selected cycle:
- let `c = pos[s]`
- swap `L[r1][c]` and `L[r2][c]`

Why it works:
- In each affected column `c`, we only swap two entries, so the column remains a permutation.
- In each affected row, the swap is structured by a cycle of `perm`, preventing duplicates and preserving permutation property.

### 3.3 Column-cycle move

Symmetric to row-cycle move, swapping columns `c1, c2` using inverse mapping on column `c1`.

### 3.4 Random cycle selection details

Implement helper:
- `fn random_nontrivial_cycle<R: Rng + ?Sized>(perm: &[u8], rng: &mut R) -> Option<Vec<u8>>`
  - Choose random start `s0`
  - Follow `perm` until repeats to identify cycle
  - If cycle length == 1, retry up to `max_retries` (e.g. 8)
  - Return `None` if not found (treat as no-op)

Note:
- `perm` can be stored as `Vec<u8>` length `n`.

## 4. Implementation details (Rust)

### 4.1 Memory / performance

- Use `Vec<u8>` for `cells`, and small stack buffers where possible.
- For each move, allocate scratch buffers:
  - `pos: Vec<usize>` length `n`
  - `perm: Vec<u8>` length `n`
  - `cycle: Vec<u8>` length up to `n`
To reduce allocations:
- Prefer `Sampler` struct holding reusable buffers (`pos`, `perm`, `visited`, etc.) if implementing iterator.

### 4.2 Index safety

- Validate `n >= 2` and `n <= 255` in public entry points.
- Use checked indexing in public methods; internal hot paths may use `get_unchecked` if well-audited (optional).

### 4.3 Floating-point probabilities

- Validate:
  - `0.0 <= p_row_move <= 1.0`
  - `0.0 <= p_do_nothing <= 1.0`
- Use `rng.gen::<f64>()` comparisons.

### 4.4 Feature flags (v0.2+)

- `serde` feature for `LatinSquare` serialization.
- `proptest` in dev-dependencies only.

## 5. Testing requirements

### 5.1 Unit tests

Use the test-only `LatinSquare::is_latin(&self) -> bool` method for validation.

1) `cyclic_is_latin`
- `LatinSquare::new_cyclic(n)` is Latin for `n=2..10`.

2) `move_preserves_latin`
- For `n=7` and `n=8`:
  - start from cyclic square
  - apply 50_000 random moves with fixed seed
  - assert `sq.is_latin()` after each move

3) `reproducibility_same_seed_same_output`
- For fixed params and seed:
  - run `sample(n, rng, params)` twice (fresh rng with same seed)
  - assert squares identical (cells equal)

4) `different_seed_different_output_smoke`
- Not a strict guarantee, but a smoke check:
  - two different seeds produce different squares (likely). If equal, allow retry with another seed.

### 5.2 Property tests (v0.2+)

Using `proptest`:
- random seeds and small `n` (2..10), random step counts (0..5000):
  - apply moves and assert `is_latin()` always true.

### 5.3 Statistical sanity checks (non-blocking / ignored tests)

Provide an ignored test (run manually):
- sample e.g. 500 squares for `n=7` with different seeds
- compute a simple statistic (e.g., count of 2x2 intercalates)
- print summary (mean/variance)
This is for human inspection only; do not fail CI.

## 6. Examples (README snippets)

### 6.1 One-shot sampling with seed

```rust
use latin_sampler::{sample, SamplerParams};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

fn main() {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let params = SamplerParams::default();
    let sq = sample(8, &mut rng, &params);
    println!("Cell (0,0) = {}", sq.get(0, 0));
}
```

### 6.2 Accessing cells

```rust
let v = sq.get(0, 0);
println!("{}", v);
```

## 7. Crate structure

### v0.1 module layout

- `src/lib.rs`
  - re-export `LatinSquare`, `SamplerParams`, `sample`
- `src/square.rs`
  - `LatinSquare` definition, constructors, accessors
  - test-only `is_latin()` helper for validation
- `src/moves.rs`
  - `row_cycle_move`, `col_cycle_move`, `random_nontrivial_cycle`
- `src/sampler.rs`
  - `SamplerParams`, `sample`

### v0.2+ additions
- `Sampler<R>` iterator in `src/sampler.rs`

## 8. Documentation expectations

Clearly state that output is "approximately uniform" and depends on mixing.
Document reproducibility guarantees (same seed + same params => same output).
Provide guidance for `n=7/8` and default params.
Warn that defaults are heuristic and may require adjustment for different `n` or strict statistical needs.

## 9. Licensing

Default recommendation for broad reuse:

- MIT OR Apache-2.0 (dual license)

Ensure dependencies (`rand`, `rand_chacha`) are compatible.

## 10. Acceptance criteria (v0.1)

- [ ] `cargo test` passes
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy` clean (no warnings)
- [ ] Reproducible sampling demonstrated in tests
- [ ] README includes the seedable example
- [ ] Public API matches Section 2.1–2.4 (excluding v0.2+ items)
- [ ] All moves preserve Latin property (no invalid intermediate states)
