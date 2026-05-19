# skyscrapers-cli

Command-line frontend for the Skyscrapers workspace. Produces the `skyscrapers` binary with two subcommands: `generate` and `solve`.

## Install / run

```bash
# Run via Cargo from the workspace root
cargo run -p skyscrapers-cli -- <subcommand> [args]

# Or install the binary on PATH
cargo install --path skyscrapers-cli
skyscrapers <subcommand> [args]
```

## `generate`

```bash
skyscrapers generate [-n <SIZE>] [--seed <SEED>] [--difficulty <LEVEL>]
```

- `-n <SIZE>` — grid size, `1..=9` (default `7`).
- `--seed <SEED>` — `u64` RNG seed; if omitted a random seed is generated and echoed to stderr.
- `--difficulty <LEVEL>` — `easy` | `medium` | `hard` | `expert` | `master`. When set, the generator retries until a puzzle whose logic-solver difficulty *exactly* matches the requested level is produced.

The puzzle is printed to stdout in the text box-format defined by `Puzzle`'s `Display` impl in [skyscrapers-core](../skyscrapers-core/README.md).

## `solve`

```bash
skyscrapers solve [FILE] [--logic]
```

- `FILE` — path to a puzzle file. If omitted, reads from stdin.
- `--logic` — use the logic solver and print a step-by-step reasoning trace instead of just the solution.

The input is parsed via `Puzzle`'s `FromStr` impl, so anything `generate` prints is valid `solve` input — which makes piping the two together the canonical smoke test:

```bash
skyscrapers generate -n 5 --seed 42 | skyscrapers solve
```
