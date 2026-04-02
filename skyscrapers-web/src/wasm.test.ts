import { describe, expect, it } from "vitest";
import { convertWasmResult } from "./wasm";

describe("convertWasmResult", () => {
  it("converts a 4x4 puzzle result", () => {
    const raw = {
      puzzle: {
        board: {
          n: 4,
          cells: [
            [null, null, null, null],
            [null, 4, null, null],
            [null, null, null, null],
            [null, null, null, null],
          ],
        },
        clues: {
          n: 4,
          top: [null, 2, null, 3],
          bottom: [null, null, 1, null],
          left: [3, null, null, null],
          right: [null, null, null, 2],
        },
      },
      solution: {
        n: 4,
        cells: [
          [2, 1, 4, 3],
          [3, 4, 1, 2],
          [4, 3, 2, 1],
          [1, 2, 3, 4],
        ],
      },
    };

    const result = convertWasmResult(raw);

    expect(result.puzzle.n).toBe(4);

    // Board cells have correct structure
    expect(result.puzzle.board[1][1]).toEqual({
      value: 4,
      given: true,
      candidates: new Set(),
    });
    expect(result.puzzle.board[0][0]).toEqual({
      value: null,
      given: false,
      candidates: new Set(),
    });

    // Clues have no `n` field
    expect(result.puzzle.clues).toEqual({
      top: [null, 2, null, 3],
      bottom: [null, null, 1, null],
      left: [3, null, null, null],
      right: [null, null, null, 2],
    });
    expect(result.puzzle.clues).not.toHaveProperty("n");

    // Solution is extracted as plain 2D array
    expect(result.solution).toEqual([
      [2, 1, 4, 3],
      [3, 4, 1, 2],
      [4, 3, 2, 1],
      [1, 2, 3, 4],
    ]);
  });
});
