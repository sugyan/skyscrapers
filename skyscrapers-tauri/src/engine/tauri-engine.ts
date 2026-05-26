import { invoke } from "@tauri-apps/api/core";
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
    const raw = await invoke<PuzzleResult>("generate_puzzle", {
      n,
      seed: seed.toString(),
      difficulty: difficulty ?? null,
    });
    return convertPuzzleResult(raw);
  }

  async requestHint(
    puzzle: Puzzle,
    board: BoardCell[][],
  ): Promise<HintResult | null> {
    // Tauri auto-maps camelCase → snake_case for command args, but pass
    // snake_case explicitly so the wire shape matches the Rust signature
    // by eye.
    const result = await invoke<HintResult | null>("next_hint", {
      puzzle: puzzleToRust(puzzle),
      board: boardToRust(board),
      user_candidates: userCandidatesToRust(board),
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
