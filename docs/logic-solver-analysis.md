# Logic Solver Analysis

Snapshot of the logic solver's behavior. Two complementary views:

1. **Unseeded baseline** (sections "Batch Test Results" and "Technique
   Usage"): what the generator produces when no target difficulty is set
   (`GeneratorParams::new(n)`). Useful for understanding the natural
   difficulty distribution of the greedy-removal pipeline.

2. **Target-driven analysis** (sections "Target Yield" and "Technique
   Necessity"): what happens when the generator is asked to produce a
   specific difficulty, and which techniques are actually load-bearing
   once a target is set. This is the more representative view for
   shipped puzzles, since end-users always pick a difficulty.

Section "Per-seed detail" contains the unseeded `batch-difficulty`
traces for reference.

## Implemented Techniques

The solver dispatches the techniques below in roughly tier order. Two
notes on dispatch ordering that aren't visible from the table:

- `XyChain` is searched before `AlsXz`. Many bivalue (size-2) ALS-XZ
  patterns are structurally length-4 XY-Chains; surfacing them as
  XyChain matches the human-friendly framing and keeps Hard-tier
  deductions from being reported as Expert.
- `SimplePermutation` (the `apply_simple` permutation pass) is
  dispatched before `AlsXz`; the heavier `PermutationEnumeration`
  (`apply_complex`) is dispatched after. The two share a code path but
  are split by `is_simple_enumeration` (≤3 free cells, or ≤8 valid
  permutations) so `AlsXz` does not shadow trivial enumerations and
  inflate the reported tier.

| Technique | Difficulty | Description |
|-----------|-----------|-------------|
| NakedSingles | Easy | Cell with one candidate |
| HiddenSingles | Easy | Value fits only one cell in line |
| CluePruning | Medium (init) | Initial candidate reduction from clues |
| VisibilityAnalysis | Medium | Clue visibility count forces monotonic prefix |
| NakedSets | Hard | k cells sharing k values |
| XWing / Swordfish | Hard | Fish pattern elimination |
| XyChain | Hard | Chain of bivalue cells whose endpoints share a value (length ≥ 3; subsumes XY-Wing) |
| SimplePermutation | Hard | Single-clue permutation check on a line trivial enough to enumerate by hand (≤3 free cells, or ≤8 valid permutations) |
| AlsXz | Expert | Two almost locked sets + restricted common candidate (size ≥ 2 — bivalue cases are absorbed by XyChain) |
| PermutationEnumeration | Expert | Single-clue permutation check on a non-trivial line |
| DualCluePermutation | Expert | Both opposing clues simultaneously |
| SimpleForcingChain | Master | Assumption + basic propagation |
| FullForcingChain | Master | Assumption + full propagation |

`SimplePermutation` is a label, not an independently dispatchable
technique: it is produced by the same permutation code path as
`PermutationEnumeration`. As a result, the `technique-necessity` tool
rejects `--disable SimplePermutation` and only lets you disable
`PermutationEnumeration` (which suppresses both labels).

## Target Yield (seeds 0-99, 100 puzzles per (size, target))

Generator success rate when a target difficulty is requested with
`max_attempts=300` per seed. Run with
`skyscrapers-analysis target-yield -n <N> --difficulty <D> --samples 100 --max-attempts 300`.

| n | easy | medium | hard | expert | master |
|---|------|--------|------|--------|--------|
| 4 | 100  | 100    | 100  | **66** | 100    |
| 5 | 100  | 100    | 100  | 100    | 100    |
| 6 | 100  | 100    | 100  | 100    | 100    |
| 7 | 100  | 100    | 100  | 100    | 100    |

The only non-100% cell is **n=4 expert (66/100)**: genuine
Expert-only 4×4 puzzles are rare because most 4×4 boards either solve
at Hard via `SimplePermutation` or escalate past Expert into Master,
leaving few that need exactly an Expert-tier technique. Every other
(n, target) combination is reliably reachable.

## Technique Necessity (target-driven, 100 puzzles per cell)

For each (n, target_difficulty), 100 puzzles were generated at the
target and re-solved with selected techniques disabled via the
`analysis-hooks` feature. Cells show: `used / harder / unsolvable`
where

- **used** = baseline solve called the disabled technique as a
  top-level step at least once. Note: techniques fired only inside
  forcing-chain propagation are not counted here, since `propagate()`
  does not emit nested steps. The `harder` and `unsolvable` columns
  reflect actual outcomes and so are unaffected by this limitation.
- **harder** = puzzle still solved but final difficulty rose
- **unsolvable** = puzzle no longer solvable by logic alone

Run with `skyscrapers-analysis technique-necessity -n <N> --difficulty <D> --samples 100 --max-attempts 500 --disable <TECH>`.

### Disable XyChain

| n | hard    | expert  | master |
|---|---------|---------|--------|
| 5 | 36/2/0  | 41/5/0  | 61/0/0 |
| 6 | 47/3/0  | 49/4/0  | 50/0/0 |
| 7 | 58/12/0 | 59/9/0  | 50/0/0 |

XyChain is invoked in 36–61% of puzzles at every (n, tier) cell, but
most of the work is substitutable: 0–12% of puzzles reclassify
upward, and zero become unsolvable. It pulls the most weight at n=7
Hard (12% bump to Expert), where `AlsXz` would otherwise have to
cover the same eliminations.

### Disable AlsXz

| n | hard    | expert  | master |
|---|---------|---------|--------|
| 5 |  4/4/0  | 30/16/0 | 38/0/0 |
| 6 | 11/11/0 | 25/9/0  | 41/0/0 |
| 7 | 10/10/0 | 33/7/0  | 56/0/0 |

AlsXz pulls real weight at Hard and Expert: every used case in n=5,6,7
Hard reclassifies upward when it's removed (4–11%), and 7–16% of
Expert puzzles also bump up. Master tier is unaffected (forcing chains
absorb the work), and no puzzle becomes unsolvable.

### Disable DualCluePermutation

| n | hard    | expert    | master    |
|---|---------|-----------|-----------|
| 5 | 0/0/0   | 13/13/0   |  6/0/0    |
| 6 | 0/0/0   |  7/7/0    | 24/0/0    |
| 7 | 0/0/0   | 16/16/0   | 30/0/**2** |

DualCluePermutation never fires on Hard puzzles (nothing to disable),
but at Expert every firing matters — 100% of n=5,6,7 expert "used"
cases reclassify upward when it's removed. At n=7 master, 2 puzzles
become unsolvable by logic alone — this is the only technique whose
removal forces a logical solve to fail.

## Batch Test Results (seeds 0-99, 100 puzzles per size)

| n | Easy | Medium | Hard | Expert | Master | Unsolved | Success |
|---|------|--------|------|--------|--------|----------|---------|
| 4 | 7    | 13     | 76   |  0     |  4     | 0        | **100%** |
| 5 | 0    |  1     | 71   |  9     | 19     | 0        | **100%** |
| 6 | 0    |  0     | 22   | 27     | 49     | 2        | **98%**  |
| 7 | 0    |  0     |  0   | 16     | 76     | 8        | **92%**  |

Under no-target generation, the modal tier shifts from Hard at n=4
(76%) through Hard/Expert/Master at n=5,6 to a Master-dominated tail
at n=7 (76% Master, 8% unsolvable). No Expert puzzle showed up at n=4
in this 100-seed slice — consistent with the 66/100 target-yield rate
above.

## Technique Usage (total step count across 100 puzzles per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 1112 | 1747 | 2465 | 3070 |
| CluePruning | 378 | 646 | 948 | 1333 |
| HiddenSingles | 263 | 523 | 773 | 1010 |
| SimplePermutation | 186 | 407 | 643 | 728 |
| VisibilityAnalysis | 51 | 105 | 113 | 100 |
| PermutationEnumeration | 1 | 68 | 362 | 1012 |
| NakedSets | 11 | 66 | 105 | 156 |
| XyChain | 25 | 71 | 74 | 91 |
| AlsXz | 3 | 30 | 53 | 73 |
| XWing | 5 | 35 | 71 | 95 |
| SimpleForcingChain | 5 | 27 | 119 | 148 |
| FullForcingChain | 1 | 21 | 78 | 289 |
| DualCluePermutation | — | 4 | 15 | 33 |

## Technique Usage (puzzles in which it appears)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 100 | 100 | 98 | 97 |
| CluePruning | 100 | 100 | 100 | 100 |
| HiddenSingles | 83 | 98 | 99 | 98 |
| SimplePermutation | 80 | 99 | 98 | 96 |
| VisibilityAnalysis | 47 | 71 | 64 | 63 |
| NakedSets | 10 | 45 | 60 | 75 |
| XyChain | 14 | 38 | 41 | 42 |
| XWing | 5 | 32 | 51 | 59 |
| AlsXz | 1 | 15 | 30 | 46 |
| PermutationEnumeration | 1 | 23 | 75 | 100 |
| SimpleForcingChain | 4 | 16 | 38 | 53 |
| FullForcingChain | 1 | 10 | 40 | 74 |
| DualCluePermutation | — | 3 | 11 | 24 |

## Observations

1. **Tier distribution is centered on Hard/Expert/Master.** Easy and
   Medium only appear at n=4 (and rarely at n=5) because the
   greedy-removal pipeline strips far enough that almost every n ≥ 5
   puzzle requires at least one Hard-tier technique. Conversely,
   Master appears in 76% of n=7 puzzles, reflecting the heavier
   reasoning the larger board admits.

2. **`SimplePermutation` is the workhorse Hard-tier technique.** It
   fires in ≥96% of puzzles at every size and is the largest
   non-singles step count at n=4..6. `PermutationEnumeration` covers
   the non-trivial cases (1% of n=4 puzzles, 100% at n=7), so the
   two-label split keeps trivial firings at Hard and lets only
   non-trivial enumerations escalate to Expert.

3. **`AlsXz` is rare at small n.** Because `SimplePermutation`
   preempts it whenever the line is trivially enumerable, `AlsXz`
   fires in only 1% of n=4 and 15% of n=5 puzzles, rising to 46% at
   n=7. It remains load-bearing where it does fire (4–16% reclassify
   upward when disabled).

4. **All Hard/Expert techniques pull weight.** Disabling `AlsXz` bumps
   4–16% of puzzles up a tier across n=5..7 Hard/Expert; disabling
   `XyChain` bumps 0–12% (no unsolvables); disabling
   `DualCluePermutation` reclassifies every Expert firing upward and
   makes 2/100 n=7 master puzzles unsolvable.

5. **`VisibilityAnalysis` is surprisingly productive at Medium tier**:
   47–71% of puzzles invoke it, and it is the reason a non-trivial
   share of n=4 puzzles register as Medium rather than Hard.

6. **Unsolvable puzzles** (no logical solve at depth 1 even with all
   techniques): 0 at n=4/5, 2 at n=6, 8 at n=7. These are the puzzles
   where even `FullForcingChain` cannot finish without nested
   assumptions.

## Per-seed detail

Per-seed `batch-difficulty` traces for the unseeded baseline (each
seed generated with `GeneratorParams::new(n)`, no target difficulty).

### n=4 Detail (seeds 0-99)

```
seed=  0  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed=  1  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed=  2  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed=  3  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed=  4  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed=  5  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed=  6  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed=  7  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed=  8  yes  easy         NakedSingles, CluePruning
seed=  9  yes  master       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 10  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 11  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 12  yes  easy         NakedSingles, CluePruning
seed= 13  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 14  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 15  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation
seed= 16  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 17  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 18  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 19  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 20  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 21  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 22  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 23  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 24  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 25  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 26  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed= 27  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 28  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed= 29  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 30  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 31  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 32  yes  hard         NakedSingles, CluePruning, XyChain, SimplePermutation
seed= 33  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, SimpleForcingChain
seed= 34  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 35  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 36  yes  hard         NakedSingles, CluePruning, NakedSets, SimplePermutation
seed= 37  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 38  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 39  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 40  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 41  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 42  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 43  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 44  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 45  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 46  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 47  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 48  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 49  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 50  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 51  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, SimpleForcingChain
seed= 52  yes  hard         NakedSingles, CluePruning, XyChain, SimplePermutation
seed= 53  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 54  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 55  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed= 56  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation
seed= 57  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 58  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 59  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 60  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 61  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 62  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 63  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 64  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 65  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 66  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 67  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 68  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 69  yes  easy         NakedSingles, CluePruning
seed= 70  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 71  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 72  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 73  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 74  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 75  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 76  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 77  yes  easy         NakedSingles, CluePruning
seed= 78  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 79  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 80  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 81  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation
seed= 82  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 83  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation
seed= 84  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 85  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 86  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed= 87  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation
seed= 88  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 89  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed= 90  yes  hard         NakedSingles, CluePruning, SimplePermutation
seed= 91  yes  master       NakedSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, SimpleForcingChain
seed= 92  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, SimplePermutation
seed= 93  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed= 94  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 95  yes  easy         NakedSingles, CluePruning
seed= 96  yes  easy         NakedSingles, CluePruning
seed= 97  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation
seed= 98  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 99  yes  easy         NakedSingles, CluePruning
```

### n=5 Detail (seeds 0-99)

```
seed=  0  yes  master       NakedSingles, HiddenSingles, CluePruning, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed=  1  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation
seed=  2  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed=  3  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed=  4  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed=  5  yes  expert       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration
seed=  6  yes  medium       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis
seed=  7  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed=  8  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed=  9  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 10  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 11  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 12  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 13  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 14  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation
seed= 15  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 16  yes  master       NakedSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 17  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 18  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, AlsXz, SimplePermutation, SimpleForcingChain
seed= 19  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 20  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 21  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 22  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 23  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 24  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 25  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 26  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation
seed= 27  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 28  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 29  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 30  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 31  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 32  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation
seed= 33  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 34  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 35  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 36  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 37  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 38  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation
seed= 39  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 40  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 41  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 42  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 43  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 44  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation, SimpleForcingChain, FullForcingChain
seed= 45  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 46  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 47  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 48  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain
seed= 49  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation
seed= 50  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 51  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation
seed= 52  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation, PermutationEnumeration
seed= 53  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 54  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 55  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation
seed= 56  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, SimpleForcingChain
seed= 57  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 58  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 59  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 60  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, PermutationEnumeration
seed= 62  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed= 63  yes  master       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 64  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 65  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration
seed= 66  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 67  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, SimpleForcingChain
seed= 68  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 69  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 70  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, SimpleForcingChain
seed= 71  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 72  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 73  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 74  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 75  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration
seed= 76  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 77  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 78  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 79  yes  master       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 80  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 81  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation
seed= 82  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation
seed= 83  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 84  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 85  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 86  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 87  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 88  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation, PermutationEnumeration
seed= 89  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed= 90  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 91  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 92  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 93  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 94  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 95  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration
seed= 96  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation
seed= 97  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 98  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 99  yes  master       NakedSingles, CluePruning, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
```

### n=6 Detail (seeds 0-99)

```
seed=  0  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed=  1  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed=  2  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  3  yes  expert       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration
seed=  4  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed=  5  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed=  6  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  7  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration
seed=  8  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  9  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 10  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration
seed= 12  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, SimpleForcingChain
seed= 13  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 14  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 15  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 16  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 17  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 18  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 19  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, SimplePermutation, FullForcingChain
seed= 20  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 21  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 22  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration
seed= 23  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration
seed= 24  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 25  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 26  yes  master       NakedSingles, HiddenSingles, CluePruning, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 27  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 28  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 29  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 30  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 31  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation
seed= 32  yes  master       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 33  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 34  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 35  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 36  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 37  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 38  yes  expert       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration, DualCluePermutation
seed= 39  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain
seed= 40  yes  expert       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration
seed= 41  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 42  yes  hard         NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, SimplePermutation
seed= 43  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 44  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 45  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 46  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 48  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 49  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 50  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, SimplePermutation, PermutationEnumeration
seed= 51  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 52  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 53  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 54  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 55  yes  hard         NakedSingles, HiddenSingles, CluePruning, SimplePermutation
seed= 56  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 57  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 58  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation
seed= 59  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, SimplePermutation, PermutationEnumeration
seed= 60  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 61  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 62  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation, SimpleForcingChain, FullForcingChain
seed= 63  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 64  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration
seed= 65  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 66  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 67  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 68  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 69  no
seed= 70  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation
seed= 71  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 72  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 73  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 74  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 75  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 76  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 77  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration
seed= 79  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed= 80  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 81  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 82  no
seed= 83  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 84  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 85  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration
seed= 86  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 87  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation
seed= 88  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 89  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, SimplePermutation, PermutationEnumeration
seed= 90  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration
seed= 91  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 92  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation
seed= 93  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation
seed= 94  yes  hard         NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, SimplePermutation
seed= 95  yes  hard         NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation
seed= 96  yes  expert       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration
seed= 97  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 98  yes  hard         NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation
seed= 99  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
```

### n=7 Detail (seeds 0-99)

```
seed=  0  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  1  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  2  yes  expert       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, SimplePermutation, PermutationEnumeration
seed=  3  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed=  4  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, FullForcingChain
seed=  5  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed=  6  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  7  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed=  8  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed=  9  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 10  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 11  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration
seed= 12  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 13  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 14  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 15  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 16  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 17  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration
seed= 18  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 19  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 20  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 21  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 22  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 23  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 24  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 25  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 26  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 27  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 28  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 29  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 30  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 31  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 32  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration
seed= 33  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 34  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 35  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 36  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 37  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 38  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 39  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 40  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 41  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 42  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 43  yes  master       NakedSingles, HiddenSingles, CluePruning, XyChain, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 44  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 45  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 46  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 47  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XyChain, SimplePermutation, PermutationEnumeration
seed= 48  no
seed= 49  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 50  no
seed= 51  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 52  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 53  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 54  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 55  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 56  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation
seed= 57  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 58  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 59  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 60  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration
seed= 61  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration
seed= 62  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 63  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, DualCluePermutation
seed= 64  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 65  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 66  no
seed= 67  no
seed= 68  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 69  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 70  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 71  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 72  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 73  no
seed= 74  no
seed= 75  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 76  yes  master       NakedSingles, HiddenSingles, CluePruning, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 77  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 78  yes  expert       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 79  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 80  yes  expert       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration
seed= 81  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration
seed= 82  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 83  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, XWing, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 84  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 85  yes  master       NakedSingles, HiddenSingles, CluePruning, XWing, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 86  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, AlsXz, SimplePermutation, PermutationEnumeration, DualCluePermutation, FullForcingChain
seed= 87  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, SimplePermutation, PermutationEnumeration, DualCluePermutation, SimpleForcingChain, FullForcingChain
seed= 88  yes  master       NakedSingles, HiddenSingles, CluePruning, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 89  no
seed= 90  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 91  no
seed= 92  yes  expert       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, AlsXz, SimplePermutation, PermutationEnumeration
seed= 93  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 94  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 95  yes  master       NakedSingles, HiddenSingles, CluePruning, NakedSets, AlsXz, SimplePermutation, PermutationEnumeration, SimpleForcingChain
seed= 96  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XWing, XyChain, SimplePermutation, PermutationEnumeration, SimpleForcingChain, FullForcingChain
seed= 97  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, NakedSets, XyChain, AlsXz, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 98  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, SimplePermutation, PermutationEnumeration, FullForcingChain
seed= 99  yes  master       NakedSingles, HiddenSingles, CluePruning, VisibilityAnalysis, XWing, SimplePermutation, PermutationEnumeration, FullForcingChain
```

## Reproduction

```bash
# Unseeded baseline (per-seed detail tables + technique usage)
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
