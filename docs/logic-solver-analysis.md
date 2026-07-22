# Logic Solver Analysis

A snapshot of the logic solver's behavior under the generator. The
numeric sections (Target Yield, Technique Necessity, Batch Test Results,
Technique Usage, Difficulty Texture) are **generated** — regenerate them
with:

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

## Difficulty Texture (n=5, 100 seeds per tier)

"Difficulty texture" looks past the headline tier at how the top-tier work is spread across the solve. Grouping the trace into maximal same-tier runs ("bursts", excluding init-only CluePruning), we report, for the top tier: **stalls** = the number of *separate* forced stalls that needed it, **topSteps** = total top-tier steps, and **longest stall** = the largest single burst. Many/long stalls with little relief feel grindier than a single hard move that unlocks an easy cascade — so two puzzles at the same tier can differ widely here. Ranking is **stalls-first** (then topSteps, then longest stall); note this puts a single long unbroken burst (1 stall but high topSteps) at the "smooth" end even though it is a long slog — how to weight stall *count* against burst *length* is an open question these numbers are meant to help settle. Use `explain` / `texture` (see Reproduction) to inspect a listed seed.

### hard

100 puzzles. Forced stalls min/median/max = 1/3/8 (stalls:count → 1:8, 2:21, 3:28, 4:22, 5:14, 6:5, 7:1, 8:1); median topSteps 6, median longest stall 2.

**Grindiest 10** (expect these to feel hardest):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 63 | 8 | 11 | 2 |
| 49 | 7 | 11 | 2 |
| 90 | 6 | 18 | 11 |
| 25 | 6 | 13 | 5 |
| 95 | 6 | 10 | 4 |
| 81 | 6 | 9 | 3 |
| 75 | 6 | 6 | 1 |
| 26 | 5 | 13 | 4 |
| 17 | 5 | 12 | 4 |
| 91 | 5 | 11 | 3 |

**Smoothest 10** (expect these to flow):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 98 | 1 | 1 | 1 |
| 53 | 1 | 1 | 1 |
| 36 | 1 | 1 | 1 |
| 28 | 1 | 1 | 1 |
| 82 | 1 | 2 | 2 |
| 41 | 1 | 7 | 7 |
| 21 | 1 | 12 | 12 |
| 7 | 1 | 12 | 12 |
| 94 | 2 | 2 | 1 |
| 76 | 2 | 2 | 1 |

_Reference — the puzzle that first motivated this metric_: `seed=20260702` → 4 stalls, 11 topSteps, longest stall 5.

### expert

100 puzzles. Forced stalls min/median/max = 1/1/4 (stalls:count → 1:77, 2:20, 3:2, 4:1); median topSteps 1, median longest stall 1.

**Grindiest 10** (expect these to feel hardest):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 99 | 4 | 9 | 4 |
| 61 | 3 | 4 | 2 |
| 46 | 3 | 3 | 1 |
| 7 | 2 | 5 | 4 |
| 72 | 2 | 4 | 3 |
| 73 | 2 | 4 | 3 |
| 79 | 2 | 4 | 3 |
| 23 | 2 | 3 | 2 |
| 25 | 2 | 3 | 2 |
| 41 | 2 | 3 | 2 |

**Smoothest 10** (expect these to flow):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 98 | 1 | 1 | 1 |
| 92 | 1 | 1 | 1 |
| 89 | 1 | 1 | 1 |
| 88 | 1 | 1 | 1 |
| 87 | 1 | 1 | 1 |
| 84 | 1 | 1 | 1 |
| 83 | 1 | 1 | 1 |
| 82 | 1 | 1 | 1 |
| 81 | 1 | 1 | 1 |
| 78 | 1 | 1 | 1 |

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

7. **Texture varies widely inside a single tier** (see Difficulty
   Texture). At n=5 Hard, forced top-tier stalls range 1–8 (median 3,
   median 6 top-tier steps), so the one "Hard" label hides a real
   spread in how grindy a puzzle feels. A single long unbroken Hard
   burst (e.g. `seed=21`: 1 stall / 12 steps) is a distinct texture from
   many short stalls (`seed=25`: 6 stalls / 13 steps); which feels
   harder is an open weighting question the section is meant to inform.
   The motivating puzzle `seed=20260702` sits at 4 stalls / 11 steps.

## Bottleneck Count (Stage 2, exploratory — reference value only)

A second within-tier signal, computed by the `bottleneck` subcommand. It
does **not** use the greedy solve trace. Instead, exploiting the fact that
every technique is a monotone, sound elimination (so any tier subset closes to
a unique fixpoint), it counts **top-tier rounds**: close the puzzle under all
techniques *below* the top tier `D`, then repeatedly (a) apply *every*
available tier-`D` deduction at once, (b) re-close under `< D`, until solved.
The number of these rounds is the **bottleneck count** — how many times the
cheap techniques stall and hard reasoning must be injected. Each round also
records a *width* (how many tier-`D` keys were available — forgiveness) and a
*cascade* (cells the following cheap closure placed).

**What it captures (validated against a single solver's felt difficulty, n=5
Hard).** The headline count cleanly separates the *smooth* Hard puzzles
(`bneck = 1`: one stall, find one of several keys, everything flows) from the
*grindy* ones (`bneck ≥ 2`). Hand-solving confirmed the easy end: `bneck = 1`
puzzles felt clearly easier, and a long forced grind of candidate-only rounds
(e.g. `seed=21`, five rounds, four of them placing nothing before a final
20-cell cascade) genuinely feels hard — so buildup rounds must be counted, not
collapsed into their eventual release.

**Ceilings (why it is a reference value, not a shippable difficulty score).**

1. *Findability noise dominates above the first bottleneck.* Whether a solver
   spots the key at a stall depends on mood/skill and varies run to run, so the
   fine ordering among `bneck = 2, 3, 4, 5` was not reliably felt in testing —
   only the `1` vs `≥ 2` split was.
2. *Not portable as an absolute value across sizes.* The scale grows with the
   grid: n=5 Hard peaks at `bneck = 1` (~50% of puzzles), but n=7 Hard peaks at
   `bneck = 3–4` with almost none at `1`. The same number means "easy" at one
   size and "typical" at another.
3. *Tier-relative, not portable across difficulties.* Because it counts waves
   of the *top* tier specifically, n=5 Expert skews to `bneck = 1` (~84%): one
   Expert insight usually unlocks a full Hard-and-below cascade. Expert and Hard
   counts are therefore not directly comparable.

**Status / future.** Kept as a dev-analysis tool only; nothing is surfaced to
players. It could become a real felt-difficulty metric with (a) solve-time data
from *multiple* solvers to average out findability noise, and (b) per-`(n,
tier)` normalization (e.g. a percentile / rank rather than a raw count). Until
then it is a within-`(n, tier)` "smooth vs grindy" relative signal, useful for
puzzle selection but not as a cross-size/cross-difficulty absolute label.

## Reproduction

```bash
# Regenerate all the generated tables above
cargo run --release -p skyscrapers-analysis -- report > /tmp/report.md

# Inspect a single puzzle's logic trace (optionally without a technique)
cargo run --release -p skyscrapers-analysis -- explain \
  -n <SIZE> --seed <SEED> --difficulty <LEVEL> [--disable <TECH>[,<TECH>...]]

# Difficulty texture: one puzzle's profile, or scan a tier for extremes
cargo run --release -p skyscrapers-analysis -- texture \
  -n <SIZE> --seed <SEED> --difficulty <LEVEL>
cargo run --release -p skyscrapers-analysis -- texture-scan \
  -n <SIZE> --difficulty <LEVEL> [--markdown --reference <SEED>]

# Bottleneck count (Stage 2, exploratory): one row per seed
cargo run --release -p skyscrapers-analysis -- bottleneck \
  -n <SIZE> -d <LEVEL> <SEED> [<SEED> ...]

# Individual sweeps (report runs all of these internally)
cargo run --release -p skyscrapers-analysis -- batch-difficulty -n <SIZE> -s <SEEDS>
cargo run --release -p skyscrapers-analysis -- target-yield -n <SIZE> --difficulty <LEVEL>
cargo run --release -p skyscrapers-analysis -- technique-necessity \
  -n <SIZE> --difficulty <LEVEL> --disable <TECH>[,<TECH>...]
```
