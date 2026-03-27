# skyscrapers-solver

Solver for Skyscrapers puzzles. Provides two implementations of the `Solver` trait.

## Solver implementations

### BacktrackingSolver

Backtracking solver with constraint propagation and MRV (Minimum Remaining Values) heuristic. Fast for small grid sizes (n≤5).

### SatSolver

SAT-based solver using [varisat](https://crates.io/crates/varisat) (CDCL SAT solver). Encodes the puzzle as a boolean satisfiability problem:

- Cell-value variables `x[r][c][v]` with Latin square constraints
- Clue constraints via permutation enumeration + Tseitin transformation
- Multiple solution enumeration via blocking clauses

Faster than BacktrackingSolver for n≥6.

## Benchmarks

`rustup run nightly cargo bench -p skyscrapers-solver`

| Solver | n=4 | n=5 | n=6 |
|---|---:|---:|---:|
| BacktrackingSolver | 2.7 µs | 41 µs | 18.3 ms |
| SatSolver | 42 µs | 174 µs | 1.2 ms |

Reference (n=7): BacktrackingSolver ~200ms, SatSolver ~24ms
