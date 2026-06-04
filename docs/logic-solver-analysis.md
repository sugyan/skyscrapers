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

`SimplePermutation` is a label, not an independently dispatchable
technique: it is produced by the same permutation code path as
`PermutationEnumeration`. The `technique-necessity`/`report` tools
therefore reject `--disable SimplePermutation` and only let you disable
`PermutationEnumeration` (which suppresses both labels).

<!-- BEGIN GENERATED (skyscrapers-analysis report) -->
## Target Yield (seeds 0-99, 100 puzzles per (size, target))

Generator success rate when a target difficulty is requested with
`max_attempts=300` per seed.

| n | easy | medium | hard | expert | master |
|---|------|--------|------|--------|--------|
| 4 | 100 | 100 | 100 | 99 | 100 |
| 5 | 100 | 100 | 100 | 100 | 100 |
| 6 | 100 | 100 | 100 | 100 | 100 |
| 7 | 100 | 100 | 100 | 100 | 100 |

## Technique Necessity (target-driven, 100 puzzles per cell)

Each cell shows `used / harder / unsolvable` for puzzles generated at
the target difficulty and re-solved with the technique disabled
(`max_attempts=500`).

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

## Batch Test Results (seeds 0-99, 100 puzzles per size)

| n | Easy | Medium | Hard | Expert | Master | Unsolved | Success |
|---|------|--------|------|--------|--------|----------|---------|
| 4 | 7 | 13 | 75 | 1 | 4 | 0 | 100% |
| 5 | 0 | 1 | 69 | 11 | 19 | 0 | 100% |
| 6 | 0 | 0 | 19 | 29 | 50 | 2 | 98% |
| 7 | 0 | 0 | 0 | 16 | 76 | 8 | 92% |

## Technique Usage (total step count across 100 puzzles per size)

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

## Technique Usage (puzzles in which it appears across 100 puzzles per size)

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
