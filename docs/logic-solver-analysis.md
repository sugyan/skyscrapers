# Logic Solver Analysis (2026-05-01)

Analysis of the logic solver's behavior. Two complementary views:

1. **Unseeded baseline** (sections "Batch Test Results" and "Technique Usage"):
   what the generator produces when no target difficulty is set
   (`GeneratorParams::new(n)`). Useful for understanding the natural
   difficulty distribution of the greedy-removal pipeline. Original numbers
   from 2026-04-24, retained as a reference point.

2. **Target-driven analysis** (sections "Target Yield" and "Technique
   Necessity"): what happens when the generator is asked to produce a
   specific difficulty, and which techniques are actually load-bearing.
   This is the more representative view for shipped puzzles, since
   end-users always pick a difficulty.

Section "Per-seed detail" contains the unseeded `batch-difficulty` traces
for reference.

## Implemented Techniques

| Technique | Difficulty | Description |
|-----------|-----------|-------------|
| NakedSingles | Easy | Cell with one candidate |
| HiddenSingles | Easy | Value fits only one cell in line |
| CluePruning | Medium (init) | Initial candidate reduction from clues |
| VisibilityAnalysis | Medium | Clue visibility count forces monotonic prefix |
| NakedSets | Hard | k cells sharing k values |
| HiddenSets | Hard | k values fitting only k cells |
| XWing / Swordfish | Hard | Fish pattern elimination |
| XYWing | Hard | Three bivalue cells pattern |
| AlsXz | Hard | Two almost locked sets + restricted common candidate |
| PermutationEnumeration | Expert | Single-clue permutation check |
| DualCluePermutation | Expert | Both opposing clues simultaneously |
| SimpleForcingChain | Master | Assumption + basic propagation |
| FullForcingChain | Grandmaster | Assumption + full propagation |

## Target Yield (seeds 0-99, 100 puzzles per (size, target))

Generator success rate when a target difficulty is requested with
`max_attempts=300` per seed. Run with
`skyscrapers-analysis target-yield -n <N> --difficulty <D> --samples 100 --max-attempts 300`.

| n | easy | medium | hard | expert | master | grandmaster |
|---|------|--------|------|--------|--------|-------------|
| 4 | 100  | 100    | 100  | 100    | 99     | **87**      |
| 5 | 100  | 100    | 100  | 100    | 100    | 100         |
| 6 | 100  | 100    | 100  | 100    | 100    | 100         |
| 7 | 100  | 100    | 100  | 100    | 100    | 100         |

Every (n, target) combination is reachable in practice. The only
non-100% cell is n=4 grandmaster (87/100): a 4×4 board is small enough
that the greedy-removal pipeline cannot always strip far enough to force
a full forcing chain. n=4 master is also borderline (99/100). Every
other category is reliably generable.

## Technique Necessity (target-driven, 100 puzzles per cell)

For each (n, target_difficulty), 100 puzzles were generated at the target
and re-solved with selected techniques disabled via the `analysis-hooks`
feature. Cells show: `used / harder / unsolvable` where
- **used** = baseline solve called the disabled technique at least once
- **harder** = puzzle still solved but final difficulty rose
- **unsolvable** = puzzle no longer solvable by logic alone

Run with `skyscrapers-analysis technique-necessity -n <N> --difficulty <D> --samples 100 --max-attempts 500 --disable <TECH>`.

### Disable XYWing

| n | hard | expert | master | grandmaster |
|---|------|--------|--------|-------------|
| 5 | 46/3/0 | 15/1/0 | 17/0/0 | 19/0/0 |
| 6 | 17/3/0 | 12/0/0 |  9/0/0 | 10/0/0 |
| 7 | 10/1/0 | 10/0/0 | 15/0/0 | 15/0/0 |

XYWing is invoked in 9–46% of puzzles but its work is almost always
substitutable. Disabling it pushes ≤3% of puzzles to a higher
difficulty label and zero puzzles become unsolvable across all 1,200
samples. Same pattern as the previously-removed W-Wing.

### Disable HiddenSets

| n | hard | expert | master | grandmaster |
|---|------|--------|--------|-------------|
| 5 |  0/0/0 |  0/0/0 |  0/0/0 |  0/0/0 |
| 6 |  3/0/0 | 14/1/0 | 15/0/0 | 24/0/0 |
| 7 |  4/0/0 | 35/0/0 | 34/0/0 | 41/0/0 |

HiddenSets fires in up to 41% of puzzles at n=7 but the work is
fully covered by other techniques: across 1,200 samples a single n=6
expert puzzle bumped Expert→Master, and zero puzzles became unsolvable.

### Disable DualCluePermutation

| n | hard | expert | master | grandmaster |
|---|------|--------|--------|-------------|
| 5 |  0/0/0 |  3/3/0 |  8/5/0 |  6/0/0 |
| 6 |  0/0/0 |  5/5/0 | 19/14/0 | 28/0/0 |
| 7 |  0/0/0 | 16/16/0 | 23/17/0 | 33/0/**4** |

DualCluePermutation pulls real weight at higher difficulties. At
n=7 master, disabling it bumps 17% of puzzles up; at n=7 grandmaster
4 puzzles become unsolvable by logic alone (forcing chain even at full
propagation cannot finish). Cannot be removed without losing solver
coverage.

### Disable XYWing + HiddenSets + DualCluePermutation together

| n | hard | expert | master | grandmaster |
|---|------|--------|--------|-------------|
| 5 | 46/3/0 | 18/4/0 | 24/5/0 | 25/0/0 |
| 6 | 20/3/0 | 27/6/0 | 38/14/0 | 49/0/0 |
| 7 | 14/1/0 | 52/16/0 | 59/17/0 | 65/0/**4** |

Effects roughly add. The 4 unsolvable cases at n=7 grandmaster all come
from DualCluePermutation specifically; XYWing and HiddenSets contribute
no unsolvable cases on their own or together.

## Batch Test Results (seeds 0-99, 100 puzzles per size)

| n | Easy | Medium | Expert | Master | Grandmaster | Unsolved | Success |
|---|------|--------|--------|--------|-------------|----------|---------|
| 4 | 7 | 13 | 76 | 3 | 1 | 0 | **100%** |
| 5 | 0 | 1 | 80 | 9 | 10 | 0 | **100%** |
| 6 | 0 | 0 | 49 | 11 | 38 | 2 | **98%** |
| 7 | 0 | 0 | 16 | 9 | 67 | 8 | **92%** |

## Technique Usage (total step count across 100 puzzles per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 1109 | 1742 | 2461 | 3085 |
| HiddenSingles | 266 | 528 | 777 | 995 |
| CluePruning | 378 | 646 | 948 | 1333 |
| VisibilityAnalysis | 51 | 106 | 111 | 97 |
| NakedSets | 11 | 62 | 103 | 158 |
| HiddenSets | — | — | 21 | 43 |
| XWing | 6 | 34 | 71 | 93 |
| XYWing | 8 | 17 | 12 | 14 |
| AlsXz | 27 | 91 | 113 | 150 |
| PermutationEnumeration | 191 | 488 | 1052 | 1786 |
| DualCluePermutation | — | 4 | 15 | 33 |
| SimpleForcingChain | 5 | 28 | 120 | 160 |
| FullForcingChain | 1 | 21 | 81 | 292 |

## Technique Usage (puzzles in which it appears)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 100 | 100 | 98 | 97 |
| HiddenSingles | 83 | 99 | 99 | 98 |
| CluePruning | 100 | 100 | 100 | 100 |
| VisibilityAnalysis | 47 | 72 | 62 | 62 |
| NakedSets | 10 | 45 | 59 | 76 |
| HiddenSets | — | — | 20 | 34 |
| XWing | 6 | 31 | 50 | 58 |
| XYWing | 6 | 15 | 10 | 13 |
| AlsXz | 18 | 45 | 58 | 66 |
| PermutationEnumeration | 80 | 99 | 100 | 100 |
| DualCluePermutation | — | 3 | 11 | 24 |
| SimpleForcingChain | 4 | 16 | 38 | 54 |
| FullForcingChain | 1 | 10 | 40 | 74 |

## Per-seed detail

Below is the original `batch-difficulty` per-seed trace from the
unseeded baseline (each seed generated with `GeneratorParams::new(n)`,
no target difficulty). Retained for reference; the headline conclusions
now come from the target-driven sections above.

### n=4 Detail (seeds 0-99)

```
seed=  0  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed=  1  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed=  2  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed=  3  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed=  4  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed=  5  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed=  6  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed=  7  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed=  8  yes  easy         NakedSingles, CluePruning
seed=  9  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 10  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 12  yes  easy         NakedSingles, CluePruning
seed= 13  yes  expert       NakedSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 14  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 15  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XYWing, PermutationEnumeration
seed= 16  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 17  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 18  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 19  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 20  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 21  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 22  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 23  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 24  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 25  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 26  yes  expert       NakedSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 27  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 28  yes  expert       NakedSingles, CluePruning, PermutationEnumeration
seed= 29  yes  expert       NakedSingles, HiddenSingles, CluePruning, XYWing, AlsXz, PermutationEnumeration
seed= 30  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 31  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 32  yes  expert       NakedSingles, CluePruning, XWing, AlsXz, PermutationEnumeration
seed= 33  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 34  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 35  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 36  yes  expert       NakedSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 37  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 38  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 39  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 40  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 41  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 42  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 43  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 44  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 45  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 46  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 48  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 49  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 50  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 51  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 52  yes  expert       NakedSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 53  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 54  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 55  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed= 56  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 57  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 58  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 59  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 60  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 62  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 63  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 64  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 65  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 66  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 67  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 68  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 69  yes  easy         NakedSingles, CluePruning
seed= 70  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, AlsXz, PermutationEnumeration
seed= 71  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 72  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 73  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 74  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 75  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 76  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 77  yes  easy         NakedSingles, CluePruning
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 79  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed= 80  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 81  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 82  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 83  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 84  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 85  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 86  yes  expert       NakedSingles, CluePruning, PermutationEnumeration
seed= 87  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration
seed= 88  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 89  yes  expert       NakedSingles, CluePruning, PermutationEnumeration
seed= 90  yes  expert       NakedSingles, CluePruning, PermutationEnumeration
seed= 91  yes  master       NakedSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration, SimpleForcingChain
seed= 92  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration
seed= 93  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 94  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 95  yes  easy         NakedSingles, CluePruning
seed= 96  yes  easy         NakedSingles, CluePruning
seed= 97  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, PermutationEnumeration
seed= 98  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 99  yes  easy         NakedSingles, CluePruning
```

### n=5 Detail (seeds 0-99)

```
seed=  0  yes  master       NakedSingles, HiddenSingles, CluePruning, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed=  1  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration
seed=  2  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, PermutationEnumeration
seed=  3  yes  expert       NakedSingles, HiddenSingles, CluePruning, XYWing, AlsXz, PermutationEnumeration
seed=  4  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed=  5  yes  expert       NakedSingles, HiddenSingles, CluePruning, XYWing, PermutationEnumeration
seed=  6  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed=  7  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XYWing, AlsXz, PermutationEnumeration
seed=  8  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed=  9  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 10  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 12  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 13  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 14  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration
seed= 15  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 16  yes  grandmaster  NakedSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 17  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 18  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 19  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 20  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 21  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 22  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 23  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 24  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 25  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 26  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 27  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 28  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 29  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 30  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 31  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 32  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration
seed= 33  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 34  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 35  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XYWing, PermutationEnumeration, FullForcingChain
seed= 36  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 37  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 38  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, PermutationEnumeration
seed= 39  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 40  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 41  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed= 42  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 43  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 44  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 45  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 46  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 48  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain
seed= 49  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, DualCluePermutation
seed= 50  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 51  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 52  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed= 53  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 54  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 55  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 56  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 57  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 58  yes  expert       NakedSingles, HiddenSingles, CluePruning, XYWing, AlsXz, PermutationEnumeration
seed= 59  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, PermutationEnumeration
seed= 60  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 62  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration
seed= 63  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 64  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 65  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 66  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 67  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration, SimpleForcingChain
seed= 68  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 69  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 70  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration, SimpleForcingChain
seed= 71  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 72  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 73  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 74  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 75  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 76  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 77  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration
seed= 79  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 80  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 81  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed= 82  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed= 83  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 84  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 85  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 86  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 87  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 88  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, PermutationEnumeration
seed= 89  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed= 90  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 91  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 92  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 93  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 94  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 95  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration
seed= 96  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 97  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 98  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 99  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
```

### n=6 Detail (seeds 0-99)

```
seed=  0  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed=  1  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed=  2  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  3  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration
seed=  4  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, FullForcingChain
seed=  5  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed=  6  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  7  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration
seed=  8  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  9  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 10  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 12  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain
seed= 13  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, FullForcingChain
seed= 14  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 15  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 16  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration
seed= 17  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 18  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 19  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 20  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 21  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 22  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 23  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 24  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 25  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 26  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, AlsXz, PermutationEnumeration, FullForcingChain
seed= 27  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration
seed= 28  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 29  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 30  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration
seed= 31  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration
seed= 32  yes  master       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 33  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 34  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 35  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 36  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 37  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration
seed= 38  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration, DualCluePermutation
seed= 39  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain
seed= 40  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 41  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 42  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration
seed= 43  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 44  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 45  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 46  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, PermutationEnumeration
seed= 48  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 49  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration, FullForcingChain
seed= 50  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, PermutationEnumeration
seed= 51  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 52  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 53  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 54  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 55  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 56  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 57  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 58  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration
seed= 59  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 60  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 62  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 63  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 64  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 65  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 66  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 67  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 68  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 69  no
seed= 70  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, AlsXz, PermutationEnumeration
seed= 71  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 72  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 73  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 74  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, PermutationEnumeration, FullForcingChain
seed= 75  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 76  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 77  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XYWing, AlsXz, PermutationEnumeration
seed= 79  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 80  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 81  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 82  no
seed= 83  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 84  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 85  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 86  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 87  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration
seed= 88  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 89  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 90  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 91  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 92  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration
seed= 93  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration
seed= 94  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration
seed= 95  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, PermutationEnumeration
seed= 96  yes  expert       NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, PermutationEnumeration
seed= 97  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 98  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration
seed= 99  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
```

### n=7 Detail (seeds 0-99)

```
seed=  0  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  1  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  2  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration
seed=  3  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed=  4  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration, FullForcingChain
seed=  5  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, FullForcingChain
seed=  6  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  7  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  8  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, FullForcingChain
seed=  9  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 10  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration
seed= 12  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 13  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain
seed= 14  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 15  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration
seed= 16  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 17  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, AlsXz, PermutationEnumeration
seed= 18  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 19  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, PermutationEnumeration, FullForcingChain
seed= 20  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 21  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration, FullForcingChain
seed= 22  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 23  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 24  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, PermutationEnumeration
seed= 25  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 26  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 27  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration, SimpleForcingChain
seed= 28  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 29  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 30  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 31  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 32  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 33  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, XYWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 34  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration, FullForcingChain
seed= 35  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 36  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 37  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 38  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 39  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 40  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 41  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 42  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 43  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, FullForcingChain
seed= 44  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 45  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 46  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration
seed= 48  no
seed= 49  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 50  no
seed= 51  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 52  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, HiddenSets, AlsXz, PermutationEnumeration, FullForcingChain
seed= 53  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 54  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 55  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 56  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation
seed= 57  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 58  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 59  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 60  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration
seed= 62  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 63  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, DualCluePermutation
seed= 64  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 65  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 66  no
seed= 67  no
seed= 68  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration
seed= 69  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 70  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 71  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XYWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 72  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 73  no
seed= 74  no
seed= 75  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 76  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 77  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, PermutationEnumeration
seed= 79  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 80  yes  expert       NakedSingles, HiddenSingles, CluePruning, PermutationEnumeration
seed= 81  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, PermutationEnumeration
seed= 82  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 83  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 84  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 85  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 86  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 87  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XYWing, AlsXz, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 88  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, HiddenSets, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 89  no
seed= 90  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 91  no
seed= 92  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, PermutationEnumeration
seed= 93  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 94  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, PermutationEnumeration, FullForcingChain
seed= 95  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, PermutationEnumeration, SimpleForcingChain
seed= 96  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 97  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, AlsXz, PermutationEnumeration, FullForcingChain
seed= 98  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, PermutationEnumeration, FullForcingChain
seed= 99  yes  grandmaster  NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, HiddenSets, XWing, AlsXz, PermutationEnumeration, FullForcingChain
```

## Observations

### From the target-driven analysis (2026-05-01)

1. **All six difficulty categories are reachable for n ≥ 5**: target yield
   is 100% across every (n, target) combination at n=5, 6, 7. n=4 starts
   to struggle at master/grandmaster (99/100 and 87/100 respectively),
   reflecting the limited room to remove cells from a 4×4 board. The
   skewed distribution in the unseeded baseline (Expert dominates at
   small n; Grandmaster at large n) is purely an artifact of the greedy
   removal — it does not mean the easier categories are impractical.

2. **XYWing is redundant**: across 1,200 target-driven puzzles, disabling
   XYWing pushed at most 3% of puzzles to a higher difficulty and zero
   puzzles became unsolvable. AlsXz absorbs its work, the same pattern
   that justified removing W-Wing.

3. **HiddenSets is redundant**: across 1,200 target-driven puzzles,
   disabling HiddenSets caused exactly one puzzle (n=6 expert) to bump up
   one tier and zero puzzles to become unsolvable. Even though it fires
   in up to 41% of puzzles at n=7, the eliminations it makes are
   reproduced by other techniques.

4. **DualCluePermutation is load-bearing**: at n=7 grandmaster,
   disabling it makes 4% of puzzles unsolvable by logic alone — i.e. no
   amount of forcing-chain propagation finishes them without the
   dual-clue lookahead. It also bumps 16–17% of n=7 expert/master puzzles
   up a tier. Cannot be removed.

5. **Forcing chains span Master and Grandmaster**: the only difference
   between the two is whether assumption-based propagation runs the
   "simple" or "full" technique pipeline. Conceptually, both are
   "guess-and-check" reasoning from the solver's perspective.

### From the unseeded baseline (2026-04-24)

1. **W-Wing removed**: Prior analysis showed W-Wing eliminations were absorbed almost entirely by ALS-XZ, so the technique was dropped. Across 400 puzzles the only behavioral change from removal was `n=6 seed=25` shifting Expert → Master.

2. **PermutationEnumeration dominates**: After NakedSingles/HiddenSingles, this is by far the most-used elimination technique at every size, and fires in ≥99% of puzzles for n ≥ 5.

3. **AlsXz picks up the W-Wing slack**: 3rd-to-5th most-used elimination technique depending on size. Per-puzzle appearance: n=4(18), n=5(45), n=6(58), n=7(66).

4. **VisibilityAnalysis is surprisingly productive**: appears in ~47–72% of puzzles and is the reason many former Expert puzzles now register as Medium at n=4.

5. **Unsolvable puzzles** (depth > 1 assumption required): 0 at n=4/5, 2 at n=6, 8 at n=7.

## Reproduction

```bash
# Unseeded baseline (per-seed detail tables)
cargo run --release -p skyscrapers-analysis -- batch-difficulty -n <SIZE> -s <SEEDS>

# Target-driven generation success rate
cargo run --release -p skyscrapers-analysis -- target-yield \
  -n <SIZE> --difficulty <LEVEL> --samples 100 --max-attempts 300

# Technique-necessity comparison (analysis-hooks feature)
cargo run --release -p skyscrapers-analysis -- technique-necessity \
  -n <SIZE> --difficulty <LEVEL> --samples 100 --max-attempts 500 \
  --disable <TECH>[,<TECH>...]

# Per-puzzle trace
cargo run --release -p skyscrapers-cli -- generate -n 7 --seed 42 \
  | cargo run --release -p skyscrapers-cli -- solve --logic
```
