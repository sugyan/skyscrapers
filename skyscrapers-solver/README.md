# skyscrapers-solver

Solver for Skyscrapers puzzles. Provides the `Solver` trait and a backtracking implementation.

## Solver implementation

### BacktrackingSolver

Backtracking solver with constraint propagation and MRV (Minimum Remaining Values) heuristic. Fast for grid sizes n≤7.

## Benchmarks

`rustup run nightly cargo bench -p skyscrapers-solver`

| Solver | n=4 | n=5 | n=6 | n=7 |
|---|---:|---:|---:|---:|
| BacktrackingSolver | 1.7 µs | 29 µs | 3.3 ms | 13.7 ms |
