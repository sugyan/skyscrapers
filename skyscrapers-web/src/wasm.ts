import init, { generate_puzzle, next_hint } from "skyscrapers-generator";
import type { BoardCell, Puzzle } from "./types";

/** Supported difficulty levels, matching the Rust `Difficulty` enum. */
export const DIFFICULTIES = [
  "easy",
  "medium",
  "hard",
  "expert",
  "master",
  "grandmaster",
] as const;

export type Difficulty = (typeof DIFFICULTIES)[number];

/**
 * Shape returned by WASM generate_puzzle (via serde-wasm-bindgen).
 * Note: serde-wasm-bindgen serializes `None` as `undefined`, not `null`.
 */
export interface WasmPuzzleResult {
  puzzle: {
    board: { n: number; cells: (number | null | undefined)[][] };
    clues: {
      n: number;
      top: (number | null | undefined)[];
      bottom: (number | null | undefined)[];
      left: (number | null | undefined)[];
      right: (number | null | undefined)[];
    };
  };
  solution: { n: number; cells: number[][] };
}

export interface GenerateResult {
  puzzle: Puzzle;
  solution: number[][];
}

export function convertWasmResult(raw: WasmPuzzleResult): GenerateResult {
  const { puzzle: wp, solution: ws } = raw;
  const puzzle: Puzzle = {
    n: wp.board.n,
    board: wp.board.cells.map((row) =>
      row.map((value) => ({
        value: value ?? null,
        given: value != null,
        candidates: new Set<number>(),
      })),
    ),
    clues: {
      top: wp.clues.top.map((v) => v ?? null),
      bottom: wp.clues.bottom.map((v) => v ?? null),
      left: wp.clues.left.map((v) => v ?? null),
      right: wp.clues.right.map((v) => v ?? null),
    },
  };
  return { puzzle, solution: ws.cells };
}

let initPromise: Promise<void> | null = null;

async function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = init().then(() => {});
  }
  await initPromise;
}

export async function generatePuzzle(
  n: number,
  seed: bigint,
  difficulty?: Difficulty,
): Promise<GenerateResult> {
  await ensureInit();
  const raw = generate_puzzle(n, seed, difficulty) as WasmPuzzleResult;
  return convertWasmResult(raw);
}

export function randomSeed(): bigint {
  return crypto.getRandomValues(new BigUint64Array(1))[0];
}

// ─── Hint API ─────────────────────────────────────────────────────────────

export type Technique =
  | "naked-singles"
  | "hidden-singles"
  | "clue-pruning"
  | "visibility-analysis"
  | "naked-sets"
  | "hidden-sets"
  | "x-wing"
  | "xy-wing"
  | "als-xz"
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
      kind: "xy-wing-elimination";
      pivot: [number, number];
      wing_a: [number, number];
      wing_b: [number, number];
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

/** Build the JSON shape Rust's `Board` expects from the user's grid. */
function boardToWasm(board: BoardCell[][]): {
  n: number;
  cells: (number | null)[][];
} {
  return {
    n: board.length,
    cells: board.map((row) => row.map((cell) => cell.value)),
  };
}

/** Build the JSON shape Rust's `Puzzle` expects from a web-shape Puzzle. */
function puzzleToWasm(puzzle: Puzzle): {
  board: { n: number; cells: (number | null)[][] };
  clues: {
    n: number;
    top: (number | null)[];
    bottom: (number | null)[];
    left: (number | null)[];
    right: (number | null)[];
  };
} {
  return {
    board: {
      n: puzzle.n,
      cells: puzzle.board.map((row) =>
        row.map((cell) => (cell.given ? cell.value : null)),
      ),
    },
    clues: {
      n: puzzle.n,
      top: puzzle.clues.top,
      bottom: puzzle.clues.bottom,
      left: puzzle.clues.left,
      right: puzzle.clues.right,
    },
  };
}

/** Extract user pencil marks as the n × n × values grid the solver expects. */
function userCandidatesToWasm(board: BoardCell[][]): number[][][] {
  return board.map((row) =>
    row.map((cell) => [...cell.candidates].sort((a, b) => a - b)),
  );
}

export async function requestHint(
  puzzle: Puzzle,
  board: BoardCell[][],
): Promise<HintResult | null> {
  await ensureInit();
  const result = next_hint(
    puzzleToWasm(puzzle),
    boardToWasm(board),
    userCandidatesToWasm(board),
  );
  return (result as HintResult | null) ?? null;
}
