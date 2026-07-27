import { useMemo } from "react";
import {
  PuzzleApp,
  normalizeDifficultyParam,
  type Difficulty,
  type PuzzleRequest,
} from "skyscrapers-player";
import { WasmEngine } from "./engine/wasm-engine";
import "skyscrapers-player/styles.css";

function parseUrlParams(): PuzzleRequest | null {
  const params = new URLSearchParams(window.location.search);
  const nStr = params.get("n");
  const seedStr = params.get("seed");
  if (!nStr || !seedStr) return null;
  const n = Number(nStr);
  if (!Number.isInteger(n) || n < 4 || n > 8) return null;
  try {
    const seed = BigInt(seedStr);
    const difficulty = normalizeDifficultyParam(params.get("difficulty"));
    return { n, seed, difficulty };
  } catch {
    return null;
  }
}

function updateUrl(n: number, seed: string, difficulty: Difficulty | null) {
  const url = new URL(window.location.href);
  url.searchParams.set("n", String(n));
  url.searchParams.set("seed", seed);
  if (difficulty) {
    url.searchParams.set("difficulty", difficulty);
  } else {
    url.searchParams.delete("difficulty");
  }
  window.history.pushState({}, "", url);
}

function clearUrl() {
  const url = new URL(window.location.href);
  url.search = "";
  window.history.pushState({}, "", url);
}

/**
 * Web host for the shared `<PuzzleApp>`: supplies the WebAssembly engine and
 * the only genuinely web-specific behavior, keeping the current puzzle's
 * parameters in the URL so a link reproduces it. Everything else — the
 * generation form, the board, How to Play — lives in `skyscrapers-player`.
 */
function App() {
  const engine = useMemo(() => new WasmEngine(), []);
  // Parsed once on first render; `PuzzleApp` only reads its initial value.
  const initialRequest = useMemo(() => parseUrlParams(), []);

  return (
    <PuzzleApp
      engine={engine}
      initialRequest={initialRequest}
      onGenerated={updateUrl}
      onCleared={clearUrl}
    />
  );
}

export default App;
