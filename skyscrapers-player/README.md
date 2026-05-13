# skyscrapers-player

Embeddable React component for playing [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles, plus a pluggable `SkyscrapersEngine` interface for generation and hint logic.

The bundled `WasmEngine` runs the Rust solver in-process via WebAssembly. A consumer that prefers to host the solver server-side can implement `SkyscrapersEngine` against their own HTTP API and pass it to `<Player>` instead.

## Status

Not published to npm. Use a `file:` reference within this monorepo for local development. A consumer-facing install path (e.g. a `player-dist` Git branch) will follow.

## Usage

```tsx
import { Player, WasmEngine, type Puzzle } from "skyscrapers-player";
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

To run the solver remotely, implement these three methods against your API and pass the instance as the `engine` prop.

## Styling

`skyscrapers-player/styles.css` is a Tailwind v4 entry that pulls in the player's CSS. Import it once at your app root. The bundled stylesheet uses `@source` to reach this package's own TSX files even when it's installed via a symlink or `node_modules`, so you don't need to extend your own Tailwind `content` paths.
