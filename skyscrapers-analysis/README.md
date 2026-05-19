# skyscrapers-analysis

Development-only tools for measuring generator and logic-solver behavior across the workspace. `publish = false` — this crate is not part of the end-user surface.

The output of these analyses backs [docs/logic-solver-analysis.md](../docs/logic-solver-analysis.md), and they exist so we can answer questions like "how often can we actually produce an Expert-rated puzzle at n=7?" or "if we disable ALS-XZ, what happens?".

## Subcommands

```bash
cargo run -p skyscrapers-analysis -- <subcommand> [args]
```

- `batch-difficulty -n <N> [--seeds <K>]` — Generate puzzles across a seed range and summarize the logic-solver difficulty distribution.
- `target-yield -n <N> --difficulty <LEVEL> [--samples <K>] [--max-attempts <M>]` — Measure how often the generator hits a target difficulty under a per-seed `max_attempts` budget.
- `technique-necessity -n <N> --difficulty <LEVEL> --disable <T1,T2,...> [--samples <K>] [--max-attempts <M>]` — For puzzles generated at a target difficulty, measure what changes when the listed techniques are disabled in the logic solver. See the source for the caveat about top-level vs. nested step counting.

## Example

Re-run the technique-necessity sweep for Hard at n=7 with `NakedSets` disabled:

```bash
cargo run -p skyscrapers-analysis --release -- technique-necessity \
  -n 7 --difficulty hard --samples 200 --disable NakedSets
```

Run `cargo run -p skyscrapers-analysis -- --help` for the full flag list on any subcommand.
