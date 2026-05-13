# skyscrapers-player

Embeddable React component for playing [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles, plus a pluggable `SkyscrapersEngine` interface for generation and hint logic.

The bundled `WasmEngine` runs the Rust solver in-process via WebAssembly. A consumer that prefers to host the solver server-side can implement `SkyscrapersEngine` against their own HTTP API and pass it to `<Player>` instead.

## Status

Not published to npm. Two install paths:

- **Inside this monorepo:** `"skyscrapers-player": "file:../skyscrapers-player"` (what `skyscrapers-web` uses).
- **From other projects:** install the `player-dist` Git branch, which is rebuilt automatically on every push to `main`:

  ```bash
  npm install github:sugyan/skyscrapers#player-dist
  ```

  The branch contains a self-contained installable package — the source under `src/`, a pre-built `dist/styles.css`, and the WebAssembly bindings vendored as a sibling directory. Your bundler (Vite, Next.js with `transpilePackages`, etc.) needs to be able to process `.ts`/`.tsx` from `node_modules`.

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
- **From the `player-dist` Git branch:** the export points at a pre-built `dist/styles.css` produced by `npm run build:css` in CI. Consumers can `import "skyscrapers-player/styles.css"` without having Tailwind in their own toolchain.
