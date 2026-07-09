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
| 4 | 100 | 100 | 100 | 99 | 100 |
| 5 | 100 | 100 | 100 | 100 | 100 |
| 6 | 100 | 100 | 100 | 100 | 100 |
| 7 | 100 | 100 | 100 | 100 | 100 |

## Technique Necessity (target-driven, 100 seeds per cell)

Each cell shows `used / harder / unsolvable` for puzzles generated at
the target difficulty and re-solved with the technique disabled
(`max_attempts=500`). Counts are over the seeds that successfully
generated a puzzle at the target — failed seeds are skipped, so the
per-cell denominator is the matching Target Yield above (every seed, for
the tiers shown here). `used` counts only techniques that surface as
top-level solve steps; a technique firing solely inside forcing-chain
propagation is not counted (the `harder`/`unsolvable` outcomes are
unaffected).

### Disable XyChain

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 13/7/0 | 26/6/0 | 22/0/0 |
| 6 | 9/1/0 | 18/0/0 | 14/0/0 |
| 7 | 18/5/0 | 20/0/0 | 13/0/0 |

### Disable AlsXz

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 0/0/0 | 58/48/0 | 48/0/0 |
| 6 | 0/0/0 | 40/22/0 | 47/0/0 |
| 7 | 0/0/0 | 44/24/0 | 66/0/0 |

### Disable DualCluePermutation

| n | hard | expert | master |
|---|------|--------|--------|
| 5 | 0/0/0 | 10/10/0 | 6/0/0 |
| 6 | 0/0/0 | 7/7/0 | 24/0/0 |
| 7 | 0/0/0 | 16/16/0 | 30/0/2 |

## Batch Test Results (seeds 0-99, 100 seeds per size)

| n | Easy | Medium | Hard | Expert | Master | Unsolved | Success |
|---|------|--------|------|--------|--------|----------|---------|
| 4 | 7 | 13 | 75 | 1 | 4 | 0 | 100% |
| 5 | 0 | 1 | 69 | 11 | 19 | 0 | 100% |
| 6 | 0 | 0 | 19 | 29 | 50 | 2 | 98% |
| 7 | 0 | 0 | 0 | 16 | 76 | 8 | 92% |

## Technique Usage (total step count across 100 seeds per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 1115 | 1769 | 2467 | 3109 |
| CluePruning | 378 | 646 | 948 | 1333 |
| HiddenSingles | 260 | 501 | 771 | 971 |
| SimplePermutation | 186 | 409 | 646 | 729 |
| VisibilityAnalysis | 51 | 105 | 112 | 100 |
| PermutationEnumeration | 1 | 68 | 361 | 1013 |
| NakedSets | 11 | 63 | 101 | 151 |
| XyChain | 11 | 18 | 13 | 17 |
| AlsXz | 4 | 45 | 63 | 111 |
| XWing | 5 | 34 | 67 | 98 |
| SimpleForcingChain | 5 | 28 | 113 | 157 |
| FullForcingChain | 1 | 21 | 78 | 281 |
| DualCluePermutation | — | 4 | 15 | 33 |

## Technique Usage (puzzles in which it appears across 100 seeds per size)

| Technique | n=4 | n=5 | n=6 | n=7 |
|-----------|-----|-----|-----|-----|
| NakedSingles | 100 | 100 | 98 | 97 |
| CluePruning | 100 | 100 | 100 | 100 |
| HiddenSingles | 83 | 98 | 99 | 98 |
| SimplePermutation | 80 | 99 | 98 | 96 |
| VisibilityAnalysis | 47 | 71 | 63 | 63 |
| PermutationEnumeration | 1 | 23 | 75 | 100 |
| NakedSets | 10 | 44 | 59 | 73 |
| XyChain | 9 | 16 | 12 | 14 |
| AlsXz | 2 | 18 | 35 | 56 |
| XWing | 5 | 31 | 49 | 62 |
| SimpleForcingChain | 4 | 16 | 38 | 55 |
| FullForcingChain | 1 | 10 | 40 | 74 |
| DualCluePermutation | — | 3 | 11 | 24 |

## Difficulty Texture (n=5, 100 seeds per tier)

"Difficulty texture" looks past the headline tier at how the top-tier work is spread across the solve. Grouping the trace into maximal same-tier runs ("bursts", excluding init-only CluePruning), we report, for the top tier: **stalls** = the number of *separate* forced stalls that needed it, **topSteps** = total top-tier steps, and **longest stall** = the largest single burst. Many/long stalls with little relief feel grindier than a single hard move that unlocks an easy cascade — so two puzzles at the same tier can differ widely here. Ranking is **stalls-first** (then topSteps, then longest stall); note this puts a single long unbroken burst (1 stall but high topSteps) at the "smooth" end even though it is a long slog — how to weight stall *count* against burst *length* is an open question these numbers are meant to help settle. Use `explain` / `texture` (see Reproduction) to inspect a listed seed.

### hard

100 puzzles. Forced stalls min/median/max = 1/3/7 (stalls:count → 1:10, 2:25, 3:37, 4:18, 5:8, 6:1, 7:1); median topSteps 5, median longest stall 3.

**Grindiest 10** (expect these to feel hardest):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 25 | 7 | 14 | 6 |
| 63 | 6 | 11 | 5 |
| 81 | 5 | 9 | 3 |
| 48 | 5 | 8 | 3 |
| 17 | 5 | 8 | 2 |
| 5 | 5 | 7 | 3 |
| 95 | 5 | 7 | 3 |
| 74 | 5 | 6 | 2 |
| 8 | 5 | 5 | 1 |
| 86 | 5 | 5 | 1 |

**Smoothest 10** (expect these to flow):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 98 | 1 | 1 | 1 |
| 36 | 1 | 1 | 1 |
| 28 | 1 | 1 | 1 |
| 53 | 1 | 2 | 2 |
| 50 | 1 | 2 | 2 |
| 82 | 1 | 3 | 3 |
| 73 | 1 | 4 | 4 |
| 41 | 1 | 6 | 6 |
| 21 | 1 | 11 | 11 |
| 7 | 1 | 12 | 12 |

_Reference — the puzzle that first motivated this metric_: `seed=20260702` → 3 stalls, 12 topSteps, longest stall 6.

### expert

100 puzzles. Forced stalls min/median/max = 1/1/5 (stalls:count → 1:67, 2:22, 3:10, 5:1); median topSteps 2, median longest stall 2.

**Grindiest 10** (expect these to feel hardest):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 99 | 5 | 10 | 4 |
| 49 | 3 | 8 | 4 |
| 70 | 3 | 7 | 4 |
| 80 | 3 | 6 | 3 |
| 46 | 3 | 5 | 3 |
| 97 | 3 | 5 | 3 |
| 11 | 3 | 5 | 2 |
| 25 | 3 | 5 | 2 |
| 45 | 3 | 4 | 2 |
| 63 | 3 | 4 | 2 |

**Smoothest 10** (expect these to flow):

| seed | stalls | topSteps | longest stall |
|------|--------|----------|---------------|
| 98 | 1 | 1 | 1 |
| 89 | 1 | 1 | 1 |
| 87 | 1 | 1 | 1 |
| 86 | 1 | 1 | 1 |
| 82 | 1 | 1 | 1 |
| 81 | 1 | 1 | 1 |
| 78 | 1 | 1 | 1 |
| 75 | 1 | 1 | 1 |
| 69 | 1 | 1 | 1 |
| 67 | 1 | 1 | 1 |

<!-- END GENERATED -->

## Observations

1. **Tier distribution is centered on Hard/Expert/Master.** Easy and
   Medium only appear at n=4 (and rarely at n=5); the greedy-removal
   pipeline strips far enough that almost every n ≥ 5 puzzle needs at
   least one Hard-tier technique. Conversely Master dominates n=7.

2. **`SimplePermutation` is the workhorse Hard-tier technique**, firing
   in ≥96% of n=5..7 puzzles. `PermutationEnumeration` covers the
   non-trivial cases (up to 100% at n=7), so the two-label split keeps
   trivial firings at Hard and lets only non-trivial enumerations
   escalate to Expert.

3. **`AlsXz` is the primary Expert-tier workhorse.** It absorbed the
   longer-chain eliminations that `XyChain` used to report, rising from
   a few % at n=4 to ~56% of n=7 puzzles, and it is load-bearing
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

7. **Texture varies widely inside a single tier** (see Difficulty
   Texture). At n=5 Hard, forced top-tier stalls range 1–7 (median 3)
   and total Hard steps 1–14, so the one "Hard" label hides a real
   spread in how grindy a puzzle feels. A single long unbroken Hard
   burst (e.g. `seed=21`: 1 stall / 11 steps) is a distinct texture from
   many short stalls (`seed=25`: 7 stalls / 14 steps); which feels
   harder is an open weighting question the section is meant to inform.
   The motivating puzzle `seed=20260702` sits at 3 stalls / 12 steps.

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
