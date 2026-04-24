# Logic Solver Analysis (2026-04-24)

Analysis of the logic solver's ability to solve generated puzzles
(unseeded difficulty, `GeneratorParams::new(n)`). Regenerated after the
W-Wing technique was removed from the solver.

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

## n=4 Detail (seeds 0-99)

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

## n=5 Detail (seeds 0-99)

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

## n=6 Detail (seeds 0-99)

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

## n=7 Detail (seeds 0-99)

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

1. **W-Wing removed**: Prior analysis showed W-Wing eliminations were absorbed almost entirely by ALS-XZ, so the technique was dropped. Across 400 puzzles the only behavioral change from removal was `n=6 seed=25` shifting Expert → Master.

2. **PermutationEnumeration dominates**: After NakedSingles/HiddenSingles, this is by far the most-used elimination technique at every size, and fires in ≥99% of puzzles for n ≥ 5.

3. **AlsXz picks up the W-Wing slack**: 3rd-to-5th most-used elimination technique depending on size. Per-puzzle appearance: n=4(18), n=5(45), n=6(58), n=7(66).

4. **VisibilityAnalysis is surprisingly productive**: appears in ~47–72% of puzzles and is the reason many former Expert puzzles now register as Medium at n=4.

5. **Unsolvable puzzles** (depth > 1 assumption required): 0 at n=4/5, 2 at n=6, 8 at n=7.

## Reproduction

```bash
# Batch test (reproduces the numbers in this doc)
cargo run --release -p skyscrapers-analysis -- batch-difficulty -n <SIZE> -s <SEEDS>

# Per-puzzle trace
cargo run --release -p skyscrapers-cli -- generate -n 7 --seed 42 \
  | cargo run --release -p skyscrapers-cli -- solve --logic
```
