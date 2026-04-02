import init, { generate_puzzle } from "skyscrapers-generator";
import type { Puzzle } from "./types";

/** Shape returned by WASM generate_puzzle (via serde-wasm-bindgen) */
interface WasmPuzzleResult {
  puzzle: {
    board: { n: number; cells: (number | null)[][] };
    clues: {
      n: number;
      top: (number | null)[];
      bottom: (number | null)[];
      left: (number | null)[];
      right: (number | null)[];
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
        value,
        given: value !== null,
        candidates: new Set<number>(),
      })),
    ),
    clues: {
      top: wp.clues.top,
      bottom: wp.clues.bottom,
      left: wp.clues.left,
      right: wp.clues.right,
    },
  };
  return { puzzle, solution: ws.cells };
}

let initialized = false;

export async function generatePuzzle(
  n: number,
  seed: bigint,
): Promise<GenerateResult> {
  if (!initialized) {
    await init();
    initialized = true;
  }
  const raw = generate_puzzle(n, seed) as WasmPuzzleResult;
  return convertWasmResult(raw);
}

export function randomSeed(): bigint {
  return crypto.getRandomValues(new BigUint64Array(1))[0];
}
