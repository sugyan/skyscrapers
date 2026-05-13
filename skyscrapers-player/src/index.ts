// Player component (the main embeddable UI surface).
export { Player } from "./components/Player";
export type { PlayerProps } from "./components/Player";

// Engine interface + shared engine types.
//
// `WasmEngine` is deliberately NOT re-exported here — importing it would
// pull `skyscrapers-generator` (the WebAssembly bindings) into the module
// graph for every consumer, even those that plug in their own remote-API
// engine. Import it from the dedicated subpath when you want it:
//
//     import { WasmEngine } from "skyscrapers-player/wasm";
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
