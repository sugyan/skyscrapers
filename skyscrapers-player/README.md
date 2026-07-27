# skyscrapers-player

Embeddable React component for playing [Skyscrapers](https://www.nikoli.co.jp/en/puzzles/skyscrapers/) pencil puzzles, plus a pluggable `SkyscrapersEngine` interface for generation and hint logic.

The package is transport-neutral: it ships the `<Player>` UI, the `SkyscrapersEngine` interface, and a small `convertPuzzleResult` helper for normalising the Rust solver's JSON output — but no engine implementation. Each consumer brings its own engine. In this repo, `skyscrapers-web` ships a `WasmEngine` that runs the Rust solver in-process via WebAssembly, and `skyscrapers-tauri` ships a `TauriEngine` that calls native Rust over IPC. A server-side deployment can implement `SkyscrapersEngine` against an HTTP API the same way.

## Status

Not published to npm. Two install paths:

- **Inside this monorepo:** `"skyscrapers-player": "file:../skyscrapers-player"` (what `skyscrapers-web` uses).
- **From other projects:** install the `player-dist` Git branch, which is rebuilt automatically on every push to `main`:

  ```bash
  npm install github:sugyan/skyscrapers#player-dist
  ```

  The branch contains a self-contained installable package: the source under `src/` and a pre-built `dist/styles.css`. Your bundler (Vite, Next.js with `transpilePackages`, etc.) needs to be able to process `.ts`/`.tsx` from `node_modules`.

## Usage

```tsx
import { Player, type Puzzle } from "skyscrapers-player";
import "skyscrapers-player/styles.css";
import { MyEngine } from "./my-engine"; // your SkyscrapersEngine implementation

const engine = new MyEngine();

function App({ puzzle, solution }: { puzzle: Puzzle; solution: number[][] }) {
  return <Player puzzle={puzzle} solution={solution} engine={engine} />;
}
```

`Player` is purely the play surface — it takes a puzzle + solution + engine and renders the grid, number pad, controls, and hint panel. Puzzle generation lives on the consumer side; call `engine.generatePuzzle(n, seed, difficulty?)` to obtain a `{ puzzle, solution }` pair.

### `PuzzleApp` — the whole app, not just the board

If you want the full experience rather than just the grid, mount `<PuzzleApp>`: it wraps `<Player>` with the generation form (size / seed / difficulty), the seed footer, and the How to Play modal, and only needs an engine.

```tsx
import { PuzzleApp } from "skyscrapers-player";

<PuzzleApp engine={engine} />;
```

This is what both `skyscrapers-web` and `skyscrapers-tauri` render, so the two stay in sync by construction. Anything platform-specific stays in the host and is wired through optional callbacks — `skyscrapers-web` uses them to keep the current puzzle's parameters in the URL:

| Prop | Purpose |
|---|---|
| `initialRequest` | A `{ n, seed, difficulty? }` to generate on first render. Only the first render's value is read. |
| `onGenerated(n, seed, difficulty)` | Fired after the user generates from the form (not for `initialRequest`). |
| `onCleared()` | Fired when the user leaves a puzzle via "New Puzzle". |

`HowToPlayModal` is exported separately for hosts that want to surface the rules outside `PuzzleApp`.

## Engine interface

```ts
interface SkyscrapersEngine {
  generatePuzzle(n, seed, difficulty?): Promise<GenerateResult>;
  requestHint(puzzle, board): Promise<HintResult | null>;
  randomSeed(): bigint;
}
```

Implement these three methods against your transport of choice (WebAssembly, Tauri IPC, HTTP) and pass the instance as the `engine` prop. The package also re-exports a transport-neutral `convertPuzzleResult(raw: PuzzleResult): GenerateResult` helper that turns the JSON shape Rust's `generate_puzzle` returns into the player's domain `Puzzle`, so your engine implementation can focus on the transport.

See `skyscrapers-web/src/engine/wasm-engine.ts` and `skyscrapers-tauri/src/engine/tauri-engine.ts` for reference implementations.

## Styling

The shape of `./styles.css` depends on how you install this package:

- **In this monorepo (`file:` link):** the export points at the Tailwind v4 source entry (`src/styles/app.css`). The consuming app must run Tailwind itself (e.g. via `@tailwindcss/vite`); it will process `@import "tailwindcss"`, the `@theme` block, and an `@source "../**/*.{ts,tsx}"` directive that walks back through the symlink so the player's classes are picked up without any extra `content` configuration on the consumer side.
- **From the `player-dist` Git branch:** the export points at a pre-built `dist/styles.css` produced by `npm run build:css` in CI. Consumers can `import "skyscrapers-player/styles.css"` without having Tailwind in their own toolchain.
