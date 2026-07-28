import type { Puzzle } from "../state/types";
import type { Difficulty, GenerateResult } from "./types";

/**
 * Transport-neutral JSON shape returned by Rust's `generate_puzzle`,
 * regardless of whether it crosses the WASM boundary (serde-wasm-bindgen)
 * or the Tauri IPC boundary (`@tauri-apps/api` `invoke`).
 *
 * `null | undefined` is accepted for optional fields because
 * serde-wasm-bindgen serializes `None` as `undefined`, while Tauri's IPC
 * serializes it as `null`.
 */
export interface PuzzleResult {
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
  /**
   * The difficulty the solver rated the generated puzzle at. Absent/`undefined`
   * (serde-wasm-bindgen serializes `None` as `undefined`) or `null` (Tauri IPC)
   * when the puzzle is harder than the logic solver can rate.
   */
  difficulty?: Difficulty | null;
}

/** Convert a raw Rust-side puzzle result into the player's domain `Puzzle`. */
export function convertPuzzleResult(raw: PuzzleResult): GenerateResult {
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
  return { puzzle, solution: ws.cells, difficulty: raw.difficulty ?? null };
}
