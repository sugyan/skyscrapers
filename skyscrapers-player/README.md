# skyscrapers-player

Embeddable React component for playing [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles, plus a pluggable `SkyscrapersEngine` interface for generation and hint logic.

The bundled `WasmEngine` runs the Rust solver in-process via WebAssembly. A consumer that prefers to host the solver server-side can implement `SkyscrapersEngine` against their own HTTP API and pass it to `<Player>` instead.

## Status

Not published to npm. Use a `file:` reference within this monorepo for local development. A consumer-facing install path (e.g. a `player-dist` Git branch) will follow.

## Usage

```tsx
import { Player, type Puzzle } from "skyscrapers-player";
import { WasmEngine } from "skyscrapers-player/wasm";
import "skyscrapers-player/styles.css";

const engine = new WasmEngine();

function App({ puzzle, solution }: { puzzle: Puzzle; solution: number[][] }) {
  return <Player puzzle={puzzle} solution={solution} engine={engine} />;
}
```

`Player` is purely the play surface — it takes a puzzle + solution + engine and renders the grid, number pad, controls, and hint panel. Puzzle generation lives on the consumer side; call `engine.generatePuzzle(n, seed, difficulty?)` to obtain a `{ puzzle, solution }` pair.

## Engine interface

```ts
interface SkyscrapersEngine {
  generatePuzzle(n, seed, difficulty?): Promise<GenerateResult>;
  requestHint(puzzle, board): Promise<HintResult | null>;
  randomSeed(): bigint;
}
```

To run the solver remotely, implement these three methods against your API and pass the instance as the `engine` prop. The `skyscrapers-player/wasm` subpath is opt-in — consumers that don't import it keep the WebAssembly bindings out of their bundle entirely.

## Styling

The shape of `./styles.css` depends on how you install this package:

- **In this monorepo (`file:` link):** the export points at the Tailwind v4 source entry (`src/styles/app.css`). The consuming app must run Tailwind itself (e.g. via `@tailwindcss/vite`); it will process `@import "tailwindcss"`, the `@theme` block, and an `@source "../**/*.{ts,tsx}"` directive that walks back through the symlink so the player's classes are picked up without any extra `content` configuration on the consumer side.
- **From the `player-dist` Git branch (planned):** the export will point at a pre-built `dist/styles.css`. Consumers will be able to `import "skyscrapers-player/styles.css"` without having Tailwind in their own toolchain. The `build:css` script generates that artifact; the branch-publish workflow (next PR) will run it and rewrite `package.json` accordingly.

Until the `player-dist` workflow lands, consuming this package from anywhere other than this repo requires Tailwind v4 in the host project.
