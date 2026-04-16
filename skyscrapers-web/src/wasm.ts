import init, { generate_puzzle } from "skyscrapers-generator";
import type { Puzzle } from "./types";

/** Supported difficulty levels, matching the Rust `Difficulty` enum. */
export type Difficulty =
  | "easy"
  | "medium"
  | "hard"
  | "expert"
  | "master"
  | "grandmaster";

export const DIFFICULTIES: readonly Difficulty[] = [
  "easy",
  "medium",
  "hard",
  "expert",
  "master",
  "grandmaster",
];

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
