import type { BoardCell, Puzzle } from "../state/types";

/** Supported difficulty levels, matching the Rust `Difficulty` enum. */
export const DIFFICULTIES = [
  "easy",
  "medium",
  "hard",
  "expert",
  "master",
] as const;

export type Difficulty = (typeof DIFFICULTIES)[number];

/**
 * Normalize a difficulty value coming from a URL/saved param.
 *
 * Returns the canonical {@link Difficulty} string, or `undefined` if the
 * input is missing or not one of {@link DIFFICULTIES}. Matching is
 * case-insensitive.
 */
export function normalizeDifficultyParam(
  raw: string | undefined | null,
): Difficulty | undefined {
  if (!raw) return undefined;
  const lower = raw.toLowerCase();
  return DIFFICULTIES.includes(lower as Difficulty)
    ? (lower as Difficulty)
    : undefined;
}

export interface GenerateResult {
  puzzle: Puzzle;
  solution: number[][];
}

export type Technique =
  | "naked-singles"
  | "hidden-singles"
  | "clue-pruning"
  | "visibility-analysis"
  | "naked-sets"
  | "x-wing"
  | "xy-chain"
  | "als-xz"
  | "prefix-permutation"
  | "simple-permutation"
  | "permutation-enumeration"
  | "dual-clue-permutation"
  | "simple-forcing-chain"
  | "full-forcing-chain";

export type Line = { row: number } | { col: number };

export type CluePosition =
  | { top: number }
  | { bottom: number }
  | { left: number }
  | { right: number };

export type HintAction =
  | { kind: "place"; row: number; col: number; value: number }
  | { kind: "eliminate"; row: number; col: number; value: number };

export type HintReason =
  | { kind: "single-candidate"; row: number; col: number }
  | { kind: "unique-in-line"; line: Line; value: number }
  | {
      kind: "set-in-line";
      line: Line;
      cells: [number, number][];
      values: number[];
    }
  | {
      kind: "fish-pattern";
      value: number;
      base_lines: Line[];
      cover_lines: Line[];
    }
  | { kind: "permutation-elimination"; line: Line; clue: CluePosition }
  | {
      kind: "dual-clue-permutation-elimination";
      line: Line;
      clue_a: CluePosition;
      clue_b: CluePosition;
    }
  | {
      kind: "xy-chain-elimination";
      chain: [number, number][];
      eliminated_value: number;
    }
  | {
      kind: "als-xz-elimination";
      als_a: [number, number][];
      als_b: [number, number][];
      rcc_value: number;
      eliminated_value: number;
    }
  | {
      kind: "forcing-chain-elimination";
      assumed_cell: [number, number];
      assumed_value: number;
    }
  | { kind: "initial-clue-constraint"; clue: CluePosition }
  | { kind: "visibility-forcing"; line: Line; clue: CluePosition };

export interface HintStep {
  technique: Technique;
  actions: HintAction[];
  reason: HintReason;
}

export interface HintResult {
  step: HintStep;
  /** n × n grid of solver-derived candidate values for each cell. */
  candidates: number[][][];
}

/**
 * Engine backing the Skyscrapers player.
 *
 * Implementations:
 *  - `WasmEngine` (in-process WebAssembly; bundled with this package)
 *  - Any user-defined adapter that proxies to a remote API
 *
 * All methods are async so a remote-API implementation can plug in without
 * the consuming UI having to know which transport it's using.
 */
export interface SkyscrapersEngine {
  generatePuzzle(
    n: number,
    seed: bigint,
    difficulty?: Difficulty,
  ): Promise<GenerateResult>;
  requestHint(puzzle: Puzzle, board: BoardCell[][]): Promise<HintResult | null>;
  randomSeed(): bigint;
}
