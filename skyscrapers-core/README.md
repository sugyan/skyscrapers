# skyscrapers-core

Shared domain types for the Skyscrapers workspace. Every other crate in the workspace builds on these types.

## Types

| Type | Description |
|---|---|
| `Solution` | A complete `n×n` grid of values in `1..=n` (a filled Latin square). |
| `Board` | An `n×n` grid where each cell is either a fixed value or empty — the player-facing surface. |
| `Clues` | The four edge clues (top, bottom, left, right), each `Option<u8>`. |
| `Puzzle` | A `Board` paired with `Clues`. Implements `Display` and `FromStr` for the text format used by the CLI. |
| `ParseError`, `SolutionParseError` | Errors returned by the corresponding `FromStr` impls. |

## Clue derivation

`Clues::from_solution(&Solution)` computes the visible-building count for each row and column from each side. A building of height `h` is visible from a given direction if no taller building stands between it and the viewer. This is the canonical way to obtain a fully-populated `Clues` for a known solution; the generator uses it as the starting point of Stage A.

## Conventions

- Cell values are **1-based** (`1..=n`).
- `Solution::new`, `Board::new_empty`, and `Clues::new_all_none` all assert `n` is in `1..=9`. The text format assumes single-digit values.
- 2D storage is `Vec<Vec<..>>`, accessed `cells[r][c]`.

## Features

- `serde` — derives `Serialize` / `Deserialize` for the public types. Used by the WASM bindings in `skyscrapers-generator`.
