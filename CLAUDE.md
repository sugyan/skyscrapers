# CLAUDE.md — latin-sampler

## 0. Purpose

`latin-sampler` is a Rust library that generates (approximately) uniformly distributed **Latin squares** of order `n` (primary target: `n = 7` or `8`) using an MCMC sampler with **seedable RNG injection** for full reproducibility.

### Non-goals
- Proving mixing time or providing a mathematically rigorous uniformity guarantee.
- Solving / enumerating all Latin squares.
- Supporting very large `n` efficiently.

### Design constraints
- Deterministic output given the same seed and parameters.
- Ergodic MCMC: must be able to reach any Latin square from any starting state.
- Simple, testable, auditable core move set.
- Reasonable performance for `n <= 10`, especially `7/8`.

### v0.1 scope (revised)
- `LatinSquare` struct with basic operations
- `SamplerParams` and `sample()` function
- **Jacobson-Matthews algorithm** for ergodic sampling
- Essential unit tests only
- Deferred to v0.2+: `Sampler<R>` iterator, `serde` feature, `proptest`

## 1. Definitions

A Latin square of order `n` is an `n x n` array with symbols `{0..n-1}` such that:
- each row is a permutation of `{0..n-1}`
- each column is a permutation of `{0..n-1}`

An **improper Latin square** (used in Jacobson-Matthews) allows one cell to have value `-1` and another to have a duplicate, with compensating `+1/-1` structure.

We treat the sampler distribution as "approximately uniform" after sufficient burn-in and mixing.

## 2. Known Issues with Previous Design

### 2.1 Problem: Ergodicity failure with row/col cycle moves

The original design used only `row_cycle_move` and `col_cycle_move`. Testing revealed:

| n | Prime? | Reduced forms reached | Total reduced forms | Coverage |
|---|--------|----------------------|---------------------|----------|
| 3 | Yes | 1 | 1 | 100% |
| 4 | No | 4 | 4 | 100% |
| 5 | Yes | **1** | 56 | **1.8%** |
| 6 | No | 8,243 | 9,408 | 87.6% |
| 7 | Yes | **1** | 16,942,080 | **~0%** |

**For prime n, the sampler was trapped in a single equivalence class.**

### 2.2 Root cause: Cyclic Latin square structure

- Cyclic square `L[r][c] = (r+c) mod n` has no intercalates when n is odd
- `row_cycle_move` and `col_cycle_move` preserve the algebraic structure
- Cannot escape to other reduced forms without additional moves

### 2.3 Solution: Jacobson-Matthews algorithm

The Jacobson-Matthews algorithm (1996) is proven to be ergodic for all n. It uses "improper" Latin squares as intermediate states, guaranteeing that any Latin square can be reached from any other.

### 2.4 Uniformity verification

Re-testing with the corrected pilot-run methodology (which accounts for non-uniform bucket probabilities) confirmed that Mode A (return-event sampling) produces uniform results for all n:

| n | χ²/df (Mode A) | Status |
|---|----------------|--------|
| 4 | 1.09 | Uniform |
| 5 | 1.02 | Uniform |
| 6 | 1.05 | Uniform |
| 7 | 0.95 | Uniform |

Earlier measurements showing non-uniformity for small n (χ²/df = 1.41 for n=4, 1.62 for n=5) were artifacts of an incorrect chi-square methodology that assumed equal bucket probabilities.

**Verification tools:**
- `examples/uniformity_test.rs`: Chi-square uniformity test with pilot-run approach
- `examples/coverage.rs`: Coverage test for small n (ergodicity verification)

## 3. Core Algorithm: Jacobson-Matthews

### 3.1 Overview

The algorithm operates on a 3-dimensional {0,1}-array representation and occasionally passes through "improper" states where one position has value -1.

Reference: Jacobson, M. T., & Matthews, P. (1996). "Generating uniformly distributed random Latin squares." *Journal of Combinatorial Designs*, 4(6), 405-437.

### 3.2 Proper Latin Square Representation

A proper Latin square can be represented as a 3D array `A[r][c][s]` where:
- `A[r][c][s] = 1` if cell (r,c) contains symbol s
- `A[r][c][s] = 0` otherwise
- For each (r,c): exactly one s has `A[r][c][s] = 1`
- For each (r,s): exactly one c has `A[r][c][s] = 1`
- For each (c,s): exactly one r has `A[r][c][s] = 1`

### 3.3 Improper Latin Square

An improper Latin square has exactly one cell (r0, c0, s0) with `A[r0][c0][s0] = -1`, and compensating `+1` entries that maintain row/column/symbol sum invariants.

### 3.4 Move Description

From a proper state:
1. Choose random (r, c, s) where `A[r][c][s] = 0`
2. Set `A[r][c][s] = +1`
3. This creates violations; find the unique (r', c', s') that restores balance
4. Set `A[r'][c'][s'] = -1`
5. If the result is proper (the -1 entry coincides with a +1), done
6. Otherwise, in improper state: choose how to resolve and iterate

### 3.5 Implementation Notes

- Can use a more compact representation (n×n array + tracking of improper state)
- Each move is O(1) average time
- Improper states are transient; algorithm quickly returns to proper states

## 4. Public API (v0.1 target)

### 4.1 Types

- `pub struct LatinSquare { n: usize, cells: Vec<u8> }`

Constraints:
- `n <= 255` (symbols stored as `u8`)
- `cells.len() == n*n`

Indexing:
- linear index `idx = r*n + c`

### 4.2 Constructors / accessors

- `impl LatinSquare`
  - `pub fn new_cyclic(n: usize) -> Self`
    - returns the cyclic Latin square: `L[r][c] = (r + c) mod n`
  - `pub fn n(&self) -> usize`
  - `pub fn get(&self, r: usize, c: usize) -> u8`
  - `pub(crate) fn set_unchecked(&mut self, r: usize, c: usize, v: u8)`

Note: `is_latin` is NOT part of the public API. A test-only helper `is_latin()` exists for validation.

### 4.3 Sampler parameters

- `pub struct SamplerParams`
  - `pub burn_in: Option<u64>`  // burn-in steps; `None` auto-scales to n³
  - `pub steps: u64`    // number of steps after burn-in before returning
  - `pub thinning: u64` // optional; if >1, only return every k steps in iterator mode
  - `pub p_do_nothing: f64` // in [0,1], for aperiodicity

Provide:
- `impl Default for SamplerParams` with safe defaults:
  - `burn_in = None` (auto-scales to n³; mixing time is empirically O(n³ log n))
  - `steps = 1_000` (reserved for iterator mode, not used in one-shot `sample()`)
  - `thinning = 1`
  - `p_do_nothing = 0.01`

Note: `p_row_move` is removed as Jacobson-Matthews does not distinguish row/col moves.

### 4.4 Sampling entry point (seedable via RNG injection)

- `pub fn sample<R: rand::Rng + ?Sized>(n: usize, rng: &mut R, params: &SamplerParams) -> LatinSquare`

RNG guidance (docs/examples):
- Recommend `rand_chacha::ChaCha20Rng` for reproducibility:
  - `let mut rng = ChaCha20Rng::from_seed([0u8; 32]);`

#### v0.2+: Sampler iterator
- `pub struct Sampler<R>` implementing `Iterator<Item = LatinSquare>`
  - created by `Sampler::new(n, rng, params)`
  - yields successive samples separated by `params.thinning`

## 5. Implementation Details (Rust)

### 5.1 Internal state representation

**Current implementation (v0.1): Full 3D array (Option A)**
- `sigma: Vec<i8>` of size `n*n*n`
- Direct implementation of the paper
- Simple and correct, but has O(n³) memory and O(n³) `is_proper()` check

Option B: Compact representation (v0.2+ optimization candidate)
- `cells: Vec<u8>` for proper state (n×n)
- `improper: Option<ImproperState>` tracking the -1 position and related info
- More memory efficient, O(1) `is_proper()` check
- Would improve performance significantly for large n

### 5.2 Memory / performance

- Use `Vec<u8>` for `cells`, and small stack buffers where possible.
- Track improper state with minimal overhead.
- Each Jacobson-Matthews move is O(1) on average.

### 5.3 Index safety

- Validate `n >= 2` and `n <= 255` in public entry points.
- Use checked indexing in public methods.

### 5.4 Feature flags (v0.2+)

- `serde` feature for `LatinSquare` serialization.
- `proptest` in dev-dependencies only.

### 5.5 Performance optimization plan

#### Approach 1: Track improper position ✅ IMPLEMENTED

The Jacobson-Matthews algorithm maintains at most one improper cell at any time. By tracking this position explicitly, both operations become O(1):

```rust
pub(crate) struct JMState {
    n: usize,
    sigma: Vec<i8>,  // keep 3D array
    improper_pos: Option<(usize, usize, usize)>,  // track -1 position
}

impl JMState {
    fn is_proper(&self) -> bool {
        self.improper_pos.is_none()  // O(1)
    }

    fn find_minus_one(&self) -> Option<(usize, usize, usize)> {
        self.improper_pos  // O(1)
    }
}
```

`apply_move()` maintains `improper_pos`:
- When setting a cell to -1: `self.improper_pos = Some((r, c, s))`
- When the -1 cell becomes 0 or 1: `self.improper_pos = None`

This is a minimal change with significant performance gain.

#### Approach 2: Compact representation (optional further optimization)

Replace 3D array with n×n cells + improper state tracking for O(n²) memory instead of O(n³):

```rust
pub(crate) struct JMState {
    n: usize,
    cells: Vec<u8>,  // n×n proper representation
    improper: Option<ImproperState>,
}

struct ImproperState {
    // Position with two symbols
    dup_row: usize,
    dup_col: usize,
    symbols: (u8, u8),  // (original, added)
    // Position with missing symbol
    missing_row: usize,
    missing_col: usize,
    missing_symbol: u8,
}
```

This requires more complex move logic but reduces memory from n³ to n² bytes.

#### Recommendation

Implement Approach 1 first (minimal change, significant gain), then evaluate if Approach 2 is needed based on profiling.

## 6. Testing Requirements

### 6.1 Unit tests

1) `cyclic_is_latin`
- `LatinSquare::new_cyclic(n)` is Latin for `n=2..10`.

2) `move_preserves_latin`
- For `n=7` and `n=8`:
  - start from cyclic square
  - apply 50_000 random Jacobson-Matthews moves with fixed seed
  - assert result `is_latin()` (proper state check)

3) `reproducibility_same_seed_same_output`
- For fixed params and seed:
  - run `sample(n, rng, params)` twice (fresh rng with same seed)
  - assert squares identical (cells equal)

4) `different_seed_different_output_smoke`
- Smoke check: two different seeds produce different squares (likely).

5) `ergodicity_reaches_multiple_reduced_forms` (new)
- For `n=5` and `n=7`:
  - sample many squares with different seeds
  - compute reduced forms
  - assert multiple distinct reduced forms are reached

### 6.2 Property tests (v0.2+)

Using `proptest`:
- random seeds and small `n` (2..10), random step counts (0..5000):
  - apply moves and assert proper state at end.

### 6.3 Statistical sanity checks (non-blocking / ignored tests)

Provide an ignored test (run manually):
- sample e.g. 500 squares for `n=7` with different seeds
- compute statistics (e.g., distribution of cell values)
- print summary (mean/variance)
This is for human inspection only; do not fail CI.

## 7. Examples (README snippets)

### 7.1 One-shot sampling with seed

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

### 7.2 Accessing cells

```rust
let v = sq.get(0, 0);
println!("{}", v);
```

## 8. Crate Structure

### v0.1 module layout

- `src/lib.rs`
  - re-export `LatinSquare`, `SamplerParams`, `sample`
- `src/square.rs`
  - `LatinSquare` definition, constructors, accessors
  - test-only `is_latin()` helper for validation
- `src/jacobson_matthews.rs` (new)
  - Jacobson-Matthews move implementation
  - Internal state management for improper states
- `src/sampler.rs`
  - `SamplerParams`, `sample`

### v0.2+ additions
- `Sampler<R>` iterator in `src/sampler.rs`
- Performance optimization (see Section 5.5 for implementation plan)

## 9. Documentation Expectations

Clearly state that output is "approximately uniform" and depends on mixing.
Document reproducibility guarantees (same seed + same params => same output).
Provide guidance for `n=7/8` and default params.
Note that Jacobson-Matthews is used for ergodicity.

## 10. Licensing

Default recommendation for broad reuse:

- MIT OR Apache-2.0 (dual license)

Ensure dependencies (`rand`, `rand_chacha`) are compatible.

## 11. Acceptance Criteria (v0.1)

- [x] `cargo test` passes
- [x] `cargo fmt --check` clean
- [x] `cargo clippy` clean (no warnings)
- [x] Reproducible sampling demonstrated in tests
- [x] README includes the seedable example
- [x] Public API matches Section 4.1–4.4 (excluding v0.2+ items)
- [x] Jacobson-Matthews algorithm correctly implemented
- [x] Ergodicity test passes (multiple reduced forms reached for n=5, n=7)

## 12. References

- Jacobson, M. T., & Matthews, P. (1996). "Generating uniformly distributed random Latin squares." *Journal of Combinatorial Designs*, 4(6), 405-437.
- McKay, B. D., & Wanless, I. M. (2005). "On the number of Latin squares." *Annals of Combinatorics*, 9(3), 335-344.
