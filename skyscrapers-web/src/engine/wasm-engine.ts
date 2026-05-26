import init, { generate_puzzle, next_hint } from "skyscrapers-generator";
import {
  computeRowColValues,
  convertPuzzleResult,
  type BoardCell,
  type Difficulty,
  type GenerateResult,
  type HintResult,
  type Puzzle,
  type PuzzleResult,
  type SkyscrapersEngine,
} from "skyscrapers-player";

let initPromise: Promise<void> | null = null;

async function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = init().then(() => {});
  }
  await initPromise;
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

/**
 * Extract user pencil marks as the n × n × values grid the solver expects.
 *
 * Values that already appear as a confirmed value elsewhere in the same row
 * or column are stripped before being sent to the solver — those candidates
 * are visually grayed-out in the UI, so the player thinks of them as already
 * eliminated. Trimming them here lets the solver's `is_step_absorbed` filter
 * skip emitting trivial "eliminate X from cell where X is already in row/col"
 * hint steps. The user's own pencil-mark state is left untouched.
 */
function userCandidatesToWasm(board: BoardCell[][]): number[][][] {
  const { rowVals, colVals } = computeRowColValues(board);
  return board.map((row, r) =>
    row.map((cell, c) =>
      [...cell.candidates]
        .filter((v) => !rowVals[r].has(v) && !colVals[c].has(v))
        .sort((a, b) => a - b),
    ),
  );
}

/** In-process Skyscrapers engine backed by the bundled WebAssembly module. */
export class WasmEngine implements SkyscrapersEngine {
  async generatePuzzle(
    n: number,
    seed: bigint,
    difficulty?: Difficulty,
  ): Promise<GenerateResult> {
    await ensureInit();
    const raw = generate_puzzle(n, seed, difficulty) as PuzzleResult;
    return convertPuzzleResult(raw);
  }

  async requestHint(
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

  randomSeed(): bigint {
    // Browsers and modern Node (≥ 19) expose Web Crypto on `globalThis.crypto`.
    // Older runtimes or unusual SSR setups may not — surface that explicitly
    // rather than letting a cryptic ReferenceError bubble up.
    const c = globalThis.crypto;
    if (!c?.getRandomValues) {
      throw new Error(
        "WasmEngine.randomSeed: globalThis.crypto.getRandomValues is unavailable. " +
          "Provide a seed argument to generatePuzzle() explicitly.",
      );
    }
    return c.getRandomValues(new BigUint64Array(1))[0];
  }
}
