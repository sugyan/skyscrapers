// Player component (the main embeddable UI surface).
export { Player } from "./components/Player";
export type { PlayerProps } from "./components/Player";

// Engine: interface + bundled WebAssembly implementation.
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
export { WasmEngine } from "./engine/wasm-engine";

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
