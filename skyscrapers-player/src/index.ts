// Player component (the main embeddable UI surface).
export { Player } from "./components/Player";
export type { PlayerProps } from "./components/Player";

// Engine interface + shared engine types.
//
// The player package is transport-neutral: it exposes the
// `SkyscrapersEngine` interface and the JSON shape Rust returns from
// `generate_puzzle`, but does NOT ship an engine implementation. Each
// consumer (web demo, Tauri app, future remote-API client) owns its own
// engine — e.g. `skyscrapers-web/src/engine/wasm-engine.ts` calls into
// the `skyscrapers-generator` WebAssembly bindings, while the Tauri app
// invokes Rust commands directly.
export type {
  SkyscrapersEngine,
  GenerateResult,
  HintResult,
  HintStep,
  HintAction,
  HintReason,
  Technique,
  Line,
  CluePosition,
  Difficulty,
} from "./engine/types";
export { DIFFICULTIES, normalizeDifficultyParam } from "./engine/types";

// Transport-neutral helpers for engines that wrap the Rust solver
// (whether over WebAssembly or Tauri IPC).
export type { PuzzleResult } from "./engine/puzzle-result";
export { convertPuzzleResult } from "./engine/puzzle-result";
export { computeRowColValues } from "./utils/board";

// Domain types consumers may need for constructing or inspecting puzzles.
export type {
  Puzzle,
  BoardCell,
  ClueValue,
  HistorySnapshot,
  InputMode,
} from "./state/types";

// Puzzle text encoding (round-trip with the CLI's compact format).
export { decodePuzzle, encodePuzzle } from "./utils/encoding";
