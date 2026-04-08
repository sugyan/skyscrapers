# Logic Solver Analysis (2026-04-08)

Analysis of the logic solver's ability to solve generated puzzles, updated after implementing ForcingChain.

## Implemented Techniques

| Technique | Difficulty |
|-----------|-----------|
| NakedSingles | Easy |
| HiddenSingles | Easy |
| CluePruning | Medium (init only) |
| NakedSets | Hard |
| HiddenSets | Hard |
| XWing / Swordfish | Hard |
| PermutationEnumeration | Expert |
| ForcingChain | Master |

## Batch Test Results (seeds 0-19)

| n | Solvable | Unsolvable | Success Rate | Previous |
|---|----------|------------|-------------|----------|
| 4 | 20/20 | 0/20 | **100%** | 85% |
| 5 | 20/20 | 0/20 | **100%** | 70% |
| 6 | 19/20 | 1/20 | **95%** | 45% |

### n=4 Detail

```
seed= 0  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 1  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 2  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 3  yes  Expert    HiddenSingles, XWing, PermutationEnumeration
seed= 4  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 5  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 6  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 7  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 8  yes  Easy
seed= 9  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed=10  yes  Expert    HiddenSingles, PermutationEnumeration
seed=11  yes  Expert    HiddenSingles, PermutationEnumeration
seed=12  yes  Easy
seed=13  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed=14  yes  Expert    HiddenSingles, PermutationEnumeration
seed=15  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed=16  yes  Expert    HiddenSingles, PermutationEnumeration
seed=17  yes  Expert    HiddenSingles, PermutationEnumeration
seed=18  yes  Expert    HiddenSingles, PermutationEnumeration
seed=19  yes  Expert    HiddenSingles, PermutationEnumeration
```

### n=5 Detail

```
seed= 0  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed= 1  yes  Expert    HiddenSingles, XWing, PermutationEnumeration
seed= 2  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed= 3  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed= 4  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed= 5  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 6  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed= 7  yes  Master    HiddenSingles, XWing, PermutationEnumeration, ForcingChain
seed= 8  yes  Expert    HiddenSingles, XWing, PermutationEnumeration
seed= 9  yes  Expert    HiddenSingles, PermutationEnumeration
seed=10  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed=11  yes  Expert    HiddenSingles, PermutationEnumeration
seed=12  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed=13  yes  Expert    HiddenSingles, NakedSets, XWing, PermutationEnumeration
seed=14  yes  Expert    HiddenSingles, NakedSets, XWing, PermutationEnumeration
seed=15  yes  Expert    HiddenSingles, PermutationEnumeration
seed=16  yes  Master    NakedSets, PermutationEnumeration, ForcingChain
seed=17  yes  Expert    HiddenSingles, NakedSets, XWing, PermutationEnumeration
seed=18  yes  Master    HiddenSingles, XWing, PermutationEnumeration, ForcingChain
seed=19  yes  Expert    HiddenSingles, PermutationEnumeration
```

### n=6 Detail

```
seed= 0  yes  Expert    HiddenSingles, PermutationEnumeration
seed= 1  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed= 2  yes  Master    HiddenSingles, NakedSets, XWing, PermutationEnumeration, ForcingChain
seed= 3  yes  Expert    HiddenSingles, XWing, PermutationEnumeration
seed= 4  yes  Master    HiddenSingles, NakedSets, PermutationEnumeration, ForcingChain
seed= 5  yes  Expert    HiddenSingles, NakedSets, PermutationEnumeration
seed= 6  yes  Master    HiddenSingles, XWing, PermutationEnumeration, ForcingChain
seed= 7  yes  Master    HiddenSingles, NakedSets, PermutationEnumeration, ForcingChain
seed= 8  yes  Master    HiddenSingles, PermutationEnumeration, ForcingChain
seed= 9  yes  Master    HiddenSingles, NakedSets, XWing, PermutationEnumeration, ForcingChain
seed=10  yes  Master    HiddenSingles, NakedSets, HiddenSets, XWing, PermutationEnumeration, ForcingChain
seed=11  yes  Expert    HiddenSingles, NakedSets, XWing, PermutationEnumeration
seed=12  yes  Master    HiddenSingles, NakedSets, PermutationEnumeration, ForcingChain
seed=13  yes  Master    HiddenSingles, NakedSets, PermutationEnumeration, ForcingChain
seed=14  yes  Expert    HiddenSingles, PermutationEnumeration
seed=15  yes  Expert    HiddenSingles, XWing, PermutationEnumeration
seed=16  yes  Expert    HiddenSingles, NakedSets, XWing, PermutationEnumeration
seed=17  no
seed=18  yes  Master    HiddenSingles, NakedSets, XWing, PermutationEnumeration, ForcingChain
seed=19  yes  Master    HiddenSingles, HiddenSets, XWing, PermutationEnumeration, ForcingChain
```

## Unsolvable Puzzle Examples

### n=6, seed=17

The only remaining unsolvable puzzle across all tested sizes and seeds.

```
    . . . 1 . .
  +-------------+
. | . . . . . . | .
2 | . . . . . . | .
. | . . . . . . | .
. | . . . . . . | .
4 | . . . . . . | 2
. | . . . . . . | .
  +-------------+
    . 5 . . . .
```
0 givens, 5 clues

## Observations

1. **ForcingChain dramatically improves success rates**: n=4 and n=5 are now 100% solvable, n=6 improved from 45% to 95%.

2. **PermutationEnumeration remains essential**: Almost all non-trivial puzzles require it. ForcingChain builds on top of it during its trial propagation.

3. **ForcingChain frequency by n**:
   - n=4: 3/20 puzzles require ForcingChain (seed 9, 13, 15)
   - n=5: 6/20 puzzles require ForcingChain (seed 0, 3, 4, 7, 16, 18)
   - n=6: 11/20 puzzles require ForcingChain — it's the dominant technique at this size

4. **HiddenSets finally appears**: In n=6, HiddenSets was used in seed=10 and seed=19 (both Master-level puzzles requiring ForcingChain). ForcingChain's trial propagation seems to create states where HiddenSets becomes relevant.

5. **Difficulty distribution**:
   - n=4: 2 Easy, 15 Expert, 3 Master
   - n=5: 0 Easy, 14 Expert, 6 Master
   - n=6: 0 Easy, 8 Expert, 11 Master, 1 Unsolvable

6. **The remaining n=6 seed=17 puzzle** has very few clues (5 clues, 0 givens). This likely requires deeper reasoning (nested forcing chains or techniques not yet implemented).

## Reproduction

```bash
# Single puzzle trace
cargo run --example trace_removal -p skyscrapers-generator -- -n 5 --seed 42

# Batch test (all seeds 0-19 for a given size)
for seed in $(seq 0 19); do
  cargo run --release --example trace_removal -p skyscrapers-generator -- -n 6 --seed $seed 2>&1 | grep "^Logic solvable:"
done
```
