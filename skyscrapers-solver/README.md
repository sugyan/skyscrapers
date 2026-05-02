# skyscrapers-solver

Solver for Skyscrapers puzzles. Provides the `Solver` trait with two implementations: a backtracking solver for fast uniqueness verification and a logic solver that uses human-like techniques for difficulty rating.

## Solver implementations

### BacktrackingSolver

Backtracking solver with constraint propagation and MRV (Minimum Remaining Values) heuristic. Used by the generator to verify solution uniqueness. Fast for grid sizes n<=7.

### LogicSolver

Applies human-traceable techniques in order of difficulty. Does not use backtracking — if no technique can make progress, the puzzle is reported as unsolvable. Returns the difficulty rating based on the hardest technique required.

#### Techniques (in application order)

| Technique | Difficulty | Description |
|---|---|---|
| NakedSingles | Easy | Cell with only one candidate remaining |
| HiddenSingles | Easy | Value that fits only one cell in a row/column |
| CluePruning | Medium | Initial candidate reduction from edge clues |
| VisibilityAnalysis | Medium | Clue visibility count forces a monotonic prefix |
| NakedSets | Hard | k cells in a line sharing exactly k candidates |
| XWing / Swordfish | Hard | Fish pattern elimination (k=2, k=3) |
| ALS-XZ | Hard | Two almost locked sets with a restricted common candidate |
| PermutationEnumeration | Expert | Enumerate valid permutations for a single clue |
| DualCluePermutation | Expert | Enumerate permutations using both opposing clues |
| SimpleForcingChain | Master | Assume a candidate, propagate with basic techniques |
| FullForcingChain | Master | Assume a candidate, propagate with all techniques |

#### Difficulty levels

- **Easy** — Naked/hidden singles only (init-time CluePruning is implicit from clue geometry and does not bump the rating)
- **Medium** — Requires visibility analysis
- **Hard** — Requires set-based, fish, or ALS techniques
- **Expert** — Requires clue permutation enumeration
- **Master** — Requires forcing-chain reasoning (assumption-based)

See [docs/logic-solver-analysis.md](../docs/logic-solver-analysis.md) for the full target-yield and technique-necessity analysis.

## Benchmarks

`rustup run nightly cargo bench -p skyscrapers-solver`

| Solver | n=4 | n=5 | n=6 | n=7 |
|---|---:|---:|---:|---:|
| BacktrackingSolver | 1.7 us | 29 us | 3.3 ms | 13.7 ms |
