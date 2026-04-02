# Skyscrapers Web

Browser-based interactive player for [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) puzzles. Built with React, TypeScript, Vite, and Tailwind CSS v4.

Puzzles can be generated directly in the browser via WASM (`skyscrapers-generator`), or selected from built-in samples.

## Puzzle Encoding Format

> **Note:** With WASM integration, the URL could use just `n` and `seed` parameters. The encoding format may still be useful for sharing specific puzzle states.

Puzzles are encoded as a compact digit string:

```
<n><top[0..n]><bottom[0..n]><left[0..n]><right[0..n]><board row-major n*n>
```

| Segment         | Length | Description                                      |
| --------------- | ------ | ------------------------------------------------ |
| `n`             | 1      | Board size as a single digit (1–9)               |
| `top[0..n]`     | n      | Top clues, left-to-right. `0` = no clue          |
| `bottom[0..n]`  | n      | Bottom clues, left-to-right                      |
| `left[0..n]`    | n      | Left clues, top-to-bottom                        |
| `right[0..n]`   | n      | Right clues, top-to-bottom                       |
| `board[0..n*n]` | n×n    | Board cells in row-major order. `0` = empty cell |

**Total length**: `1 + 4n + n²`

| n   | Total length |
| --- | ------------ |
| 5   | 46           |
| 7   | 78           |
| 8   | 97           |

### Clue Ordering

The clue order matches the Rust `Clues` struct: **top, bottom, left, right**.

### Example

For an n=5 puzzle:

```
500300010030000003040200000000000000000000000020000
^     ^     ^     ^     ^
n  top[5] bot[5] lft[5] rgt[5]  board[25]
```

## Development

The WASM package must be built before running the web app:

```bash
# Build WASM package (requires wasm-pack)
wasm-pack build --target web skyscrapers-generator

# Web app commands
cd skyscrapers-web
npm install
npm run dev            # Start dev server
npm run build          # Type-check and build for production
npm run preview        # Preview production build
npm run lint           # Run ESLint
npm run format:check   # Check formatting with Prettier
npm run test           # Run tests with Vitest
```

## Future Work

- **Web Worker**: For larger board sizes (n=7, 8), puzzle generation can take several seconds and block the UI. Moving the WASM call to a Web Worker would keep the UI responsive during generation.
