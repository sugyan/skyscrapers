import { invoke } from "@tauri-apps/api/core";
import { convertWasmResult, type WasmPuzzleResult } from "skyscrapers-player/wasm";
import type {
  BoardCell,
  Difficulty,
  GenerateResult,
  HintResult,
  Puzzle,
  SkyscrapersEngine,
} from "skyscrapers-player";

/** Build the JSON shape Rust's `Puzzle` expects from a player-shape Puzzle. */
function puzzleToRust(puzzle: Puzzle) {
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

function boardToRust(board: BoardCell[][]) {
  return {
    n: board.length,
    cells: board.map((row) => row.map((cell) => cell.value)),
  };
}

function computeRowColValues(board: BoardCell[][]): {
  rowVals: Set<number>[];
  colVals: Set<number>[];
} {
  const n = board.length;
  const rowVals: Set<number>[] = Array.from({ length: n }, () => new Set());
  const colVals: Set<number>[] = Array.from({ length: n }, () => new Set());
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      const v = board[r][c].value;
      if (v !== null) {
        rowVals[r].add(v);
        colVals[c].add(v);
      }
    }
  }
  return { rowVals, colVals };
}

function userCandidatesToRust(board: BoardCell[][]): number[][][] {
  const { rowVals, colVals } = computeRowColValues(board);
  return board.map((row, r) =>
    row.map((cell, c) =>
      [...cell.candidates]
        .filter((v) => !rowVals[r].has(v) && !colVals[c].has(v))
        .sort((a, b) => a - b),
    ),
  );
}

/** Native Skyscrapers engine backed by Tauri commands (Rust in the host process). */
export class TauriEngine implements SkyscrapersEngine {
  async generatePuzzle(
    n: number,
    seed: bigint,
    difficulty?: Difficulty,
  ): Promise<GenerateResult> {
    const raw = await invoke<WasmPuzzleResult>("generate_puzzle", {
      n,
      seed: seed.toString(),
      difficulty: difficulty ?? null,
    });
    return convertWasmResult(raw);
  }

  async requestHint(
    puzzle: Puzzle,
    board: BoardCell[][],
  ): Promise<HintResult | null> {
    const result = await invoke<HintResult | null>("next_hint", {
      puzzle: puzzleToRust(puzzle),
      board: boardToRust(board),
      userCandidates: userCandidatesToRust(board),
    });
    return result;
  }

  randomSeed(): bigint {
    const c = globalThis.crypto;
    if (!c?.getRandomValues) {
      throw new Error(
        "TauriEngine.randomSeed: globalThis.crypto.getRandomValues is unavailable.",
      );
    }
    return c.getRandomValues(new BigUint64Array(1))[0];
  }
}
