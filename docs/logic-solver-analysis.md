# Logic Solver Analysis

A snapshot of the logic solver's behavior under the generator. The
numeric sections (Target Yield, Technique Necessity, Batch Test Results,
Technique Usage) are **generated** — regenerate them with:

```bash
cargo run -p skyscrapers-analysis --release -- report > /tmp/report.md
```

and splice the sections below. The "Implemented Techniques" and
"Observations" sections are hand-written and updated alongside. (Per-seed
dumps were intentionally dropped; use `explain` to inspect a single
puzzle — see Reproduction.)

Two complementary views:

1. **Unseeded baseline** ("Batch Test Results", "Technique Usage"): what
   the generator produces with no target difficulty
   (`GeneratorParams::new(n)`) — the natural difficulty distribution of
   the greedy-removal pipeline.
2. **Target-driven** ("Target Yield", "Technique Necessity"): what
   happens when a specific difficulty is requested, and which techniques
   are load-bearing. The more representative view for shipped puzzles.

## Implemented Techniques

The solver dispatches the techniques below in roughly tier order. Two
notes on dispatch ordering that aren't visible from the table:

- `XyChain` is capped at length 3, i.e. it only finds the classic
  XY-Wing. It is searched before `AlsXz` so genuine XY-Wings are
  credited as Hard, but longer bivalue chains are *not* searched here:
  they are structurally equivalent to size-2 ALS-XZ patterns and need
  the same "if this then … then contradiction" mental trace as a
  forcing chain, so they are deliberately left to `AlsXz` and reported
  at Expert.
- `SimplePermutation` (the `apply_simple` permutation pass) is
  dispatched before `AlsXz`; the heavier `PermutationEnumeration`
  (`apply_complex`) is dispatched after. The two share a code path but
  are split by `is_simple_enumeration` (≤3 free cells, or ≤8 valid
  permutations) so `AlsXz` does not shadow trivial enumerations and
  inflate the reported tier.
- `PrefixPermutation` is dispatched just before `SimplePermutation`. It
  is a cheap, suffix-ignoring subset of permutation enumeration, so it
  adds no solving power over `PermutationEnumeration`; its purpose is to
  claim the forward clue-only eliminations a human makes at Hard, keeping
  them from being absorbed by the Expert-tier `PermutationEnumeration` /
  `AlsXz` passes.

Difficulty is the tier of the hardest technique the solve actually
needs, ranked by `Technique::difficulty()` (not by enum declaration
order), so an Expert technique can never be masked by a lower-tier one.

| Technique | Difficulty | Description |
|-----------|-----------|-------------|
| NakedSingles | Easy | Cell with one candidate |
| HiddenSingles | Easy | Value fits only one cell in line |
| CluePruning | Medium (init) | Initial candidate reduction from clues |
| VisibilityAnalysis | Medium | Clue visibility count forces monotonic prefix |
| NakedSets | Hard | k cells sharing k values |
| XWing / Swordfish | Hard | Fish pattern elimination |
| XyChain | Hard | XY-Wing: a length-3 bivalue chain (pivot + two wings). Longer chains are not searched — they fall through to AlsXz |
| PrefixPermutation | Hard | Forward (prefix-only) visibility: the cells up to the target fix how many buildings are visible so far; values that make the clue's count unreachable are eliminated. A cheap, suffix-ignoring subset of permutation enumeration |
| SimplePermutation | Hard | Single-clue permutation check on a line trivial enough to enumerate by hand (≤3 free cells, or ≤8 valid permutations) |
| AlsXz | Expert | Two almost locked sets + restricted common candidate (size ≥ 2). Includes the size-2 ALS patterns that longer bivalue chains form, now that XyChain is capped at the XY-Wing |
| PermutationEnumeration | Expert | Single-clue permutation check on a non-trivial line |
| DualCluePermutation | Expert | Both opposing clues simultaneously |
| SimpleForcingChain | Master | Assumption + basic propagation |
| FullForcingChain | Master | Assumption + full propagation |

`SimplePermutation` and `PermutationEnumeration` are two separate
dispatchable techniques backed by the same `permutation` module:
`apply_simple` (trivial lines — ≤3 free cells, or ≤8 valid permutations)
is reported as `SimplePermutation` (Hard), and `apply_complex`
(non-trivial lines) as `PermutationEnumeration` (Expert). Either can be
disabled independently via `--disable`.

<!-- BEGIN GENERATED (skyscrapers-analysis report) -->
## Target Yield (seeds 0-99, 100 seeds per (size, target))

Generator success rate when a target difficulty is requested with
`max_attempts=300` per seed.

| n | easy | medium | hard | expert | master |
|---|------|--------|------|--------|--------|
| 4 | 100 | 100 | 100 | 98 | 100 |
| 5 | 100 | 100 | 100 | 100 | 100 |
| 6 | 100 | 100 | 100 | 100 | 100 |
| 7 | 100 | 100 | 100 | 100 | 100 |

## Technique Necessity (target-driven, 100 seeds per cell)

Each cell shows `used / harder / unsolvable` for puzzles generated at
the target difficulty and re-solved with the technique disabled
(`max_attempts=500`). Counts are over the seeds that
successfully generated a puzzle at the target — failed seeds are skipped,
so the per-cell denominator is the matching Target Yield above (every
seed, for the tiers shown here). `used` counts only techniques that
surface as top-level solve steps; a technique firing solely inside
forcing-chain propagation is not counted (the `harder`/`unsolvable`
outcomes are unaffected).

### Disable XyChain

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 10/7/0 | 30/6/0 | 20/0/0 |
| 6 | 7/2/0 | 22/0/0 | 15/0/0 |
| 7 | 14/6/0 | 19/0/0 | 13/0/0 |

### Disable AlsXz

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 0/0/0 | 78/72/0 | 48/0/0 |
| 6 | 0/0/0 | 52/31/0 | 49/0/0 |
| 7 | 0/0/0 | 42/24/0 | 63/0/1 |

### Disable DualCluePermutation

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 0/0/0 | 13/13/0 | 6/0/0 |
| 6 | 0/0/0 | 11/11/0 | 24/0/0 |
| 7 | 0/0/0 | 17/17/0 | 31/0/3 |

## Batch Test Results (seeds 0-99, 100 seeds per size)

| n | Easy | Medium | Hard | Expert | Master | Unsolved | Success |
|---|------|--------|------|--------|--------|----------|---------|
| 4 | 7 | 13 | 75 | 1 | 4 | 0 | 100% |
| 5 | 0 | 1 | 72 | 8 | 19 | 0 | 100% |
| 6 | 0 | 0 | 28 | 20 | 50 | 2 | 98% |
| 7 | 0 | 0 | 0 | 16 | 75 | 9 | 91% |

## Technique Usage (total step count across 100 seeds per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 1075 | 1657 | 2317 | 2921 |
| CluePruning | 378 | 646 | 948 | 1333 |
| HiddenSingles | 300 | 613 | 921 | 1122 |
| PrefixPermutation | 168 | 307 | 623 | 790 |
| SimplePermutation | 33 | 199 | 375 | 526 |
| VisibilityAnalysis | 51 | 107 | 116 | 99 |
| PermutationEnumeration | — | 27 | 223 | 748 |
| NakedSets | 11 | 70 | 116 | 149 |
| XyChain | 9 | 16 | 13 | 16 |
| AlsXz | 4 | 44 | 63 | 100 |
| XWing | 4 | 33 | 74 | 103 |
| SimpleForcingChain | 5 | 28 | 118 | 143 |
| FullForcingChain | 1 | 20 | 75 | 274 |
| DualCluePermutation | — | 4 | 15 | 35 |

## Technique Usage (puzzles in which it appears across 100 seeds per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 100 | 100 | 98 | 97 |
| CluePruning | 100 | 100 | 100 | 100 |
| HiddenSingles | 85 | 99 | 99 | 97 |
| PrefixPermutation | 78 | 98 | 100 | 99 |
| SimplePermutation | 23 | 79 | 95 | 94 |
| VisibilityAnalysis | 47 | 72 | 63 | 61 |
| PermutationEnumeration | — | 16 | 65 | 98 |
| NakedSets | 10 | 46 | 67 | 78 |
| XyChain | 7 | 15 | 11 | 14 |
| AlsXz | 2 | 18 | 37 | 53 |
| XWing | 4 | 30 | 51 | 66 |
| SimpleForcingChain | 4 | 16 | 39 | 52 |
| FullForcingChain | 1 | 10 | 40 | 73 |
| DualCluePermutation | — | 3 | 11 | 25 |
<!-- END GENERATED -->

## Observations

1. **Tier distribution is centered on Hard/Expert/Master.** Easy and
   Medium only appear at n=4 (and rarely at n=5); the greedy-removal
   pipeline strips far enough that almost every n ≥ 5 puzzle needs at
   least one Hard-tier technique. Conversely Master dominates n=7.

2. **The permutation family is the Hard-tier workhorse, now split three
   ways.** `PrefixPermutation` — the cheap forward (prefix-only)
   deduction — fires in 78–100% of puzzles and claims most single-clue
   eliminations at Hard. `SimplePermutation` mops up the residual trivial
   lines it doesn't (23% at n=4, ~79–95% at n=5..7), and
   `PermutationEnumeration` handles the non-trivial enumerations that
   genuinely need Expert (up to ~98% at n=7). The three-label split keeps
   forward and trivial clue reasoning at Hard and lets only non-trivial
   enumeration escalate to Expert — puzzles whose only "Expert" step was
   actually a forward clue deduction (e.g. n=4 seed=20260605) now rate
   Hard.

3. **`AlsXz` is the primary Expert-tier workhorse.** It absorbed the
   longer-chain eliminations that `XyChain` used to report, rising from
   a few % at n=4 to ~53% of n=7 puzzles, and it is load-bearing
   wherever it fires (a large share of Expert puzzles reclassify upward
   when it is disabled). It never appears in Hard puzzles, confirming
   the tier separation.

4. **`XyChain` is bounded to the XY-Wing.** With the length-3 cap it has
   a small footprint; the longer bivalue chains it used to find now
   surface as `AlsXz` at Expert, matching their forcing-chain-like
   solving effort.

5. **`VisibilityAnalysis` is productive at Medium tier** (invoked by a
   large fraction of puzzles) and is the main reason some n=4 puzzles
   register as Medium rather than Hard.

6. **Unsolvable puzzles** (no logical solve at depth 1 even with all
   techniques) appear only at n=6 (a couple) and n=7 (a handful) — the
   puzzles where even `FullForcingChain` cannot finish without nested
   assumptions.

   The depth-1, non-recursive forcing chains are mildly **order-sensitive**:
   they only inspect 2–3 candidate cells, so a sound but *earlier*
   elimination (e.g. from `PrefixPermutation` at Hard, before the chains
   run) can reshape the mid-solve candidate structure enough that the
   specific assumption which would have cracked a borderline puzzle is no
   longer reachable at depth 1. Introducing `PrefixPermutation` nudged one
   n=7 baseline puzzle (seed 87) across this boundary — it now reports
   Unsolved where it previously finished at Master. This is a completeness
   artifact of the depth-1 solver, not a soundness issue: every step
   remains sound, the puzzle stays uniquely solvable (the backtracking
   solver finds its single solution), and shipped puzzles are unaffected —
   target-difficulty generation only accepts puzzles the logic solver
   fully solves (`is_removal_ok`).

## Reproduction

```bash
# Regenerate all the generated tables above
cargo run --release -p skyscrapers-analysis -- report > /tmp/report.md

# Inspect a single puzzle's logic trace (optionally without a technique)
cargo run --release -p skyscrapers-analysis -- explain \
  -n <SIZE> --seed <SEED> --difficulty <LEVEL> [--disable <TECH>[,<TECH>...]]

# Individual sweeps (report runs all of these internally)
cargo run --release -p skyscrapers-analysis -- batch-difficulty -n <SIZE> -s <SEEDS>
cargo run --release -p skyscrapers-analysis -- target-yield -n <SIZE> --difficulty <LEVEL>
cargo run --release -p skyscrapers-analysis -- technique-necessity \
  -n <SIZE> --difficulty <LEVEL> --disable <TECH>[,<TECH>...]
```
