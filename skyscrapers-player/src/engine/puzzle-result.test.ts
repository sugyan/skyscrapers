import { describe, expect, it } from "vitest";
import { convertPuzzleResult } from "./puzzle-result";

describe("convertPuzzleResult", () => {
  it("converts a 4x4 puzzle result with null values", () => {
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

    const result = convertPuzzleResult(raw);

    expect(result.puzzle.n).toBe(4);
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
    expect(result.puzzle.clues).toEqual({
      top: [null, 2, null, 3],
      bottom: [null, null, 1, null],
      left: [3, null, null, null],
      right: [null, null, null, 2],
    });
    expect(result.puzzle.clues).not.toHaveProperty("n");
    expect(result.solution).toEqual([
      [2, 1, 4, 3],
      [3, 4, 1, 2],
      [4, 3, 2, 1],
      [1, 2, 3, 4],
    ]);
  });

  it("normalizes undefined to null (serde-wasm-bindgen behavior)", () => {
    // serde-wasm-bindgen serializes None as undefined, not null. The Tauri
    // IPC boundary serializes it as null. The converter accepts both.
    const raw = {
      puzzle: {
        board: {
          n: 3,
          cells: [
            [undefined, 2, undefined],
            [undefined, undefined, undefined],
            [3, undefined, undefined],
          ],
        },
        clues: {
          n: 3,
          top: [undefined, 1, undefined],
          bottom: [undefined, undefined, 2],
          left: [undefined, undefined, undefined],
          right: [3, undefined, undefined],
        },
      },
      solution: {
        n: 3,
        cells: [
          [1, 2, 3],
          [2, 3, 1],
          [3, 1, 2],
        ],
      },
    };

    const result = convertPuzzleResult(raw);

    expect(result.puzzle.board[0][0]).toEqual({
      value: null,
      given: false,
      candidates: new Set(),
    });
    expect(result.puzzle.board[0][1]).toEqual({
      value: 2,
      given: true,
      candidates: new Set(),
    });
    expect(result.puzzle.board[2][0]).toEqual({
      value: 3,
      given: true,
      candidates: new Set(),
    });

    expect(result.puzzle.clues.top).toEqual([null, 1, null]);
    expect(result.puzzle.clues.bottom).toEqual([null, null, 2]);
    expect(result.puzzle.clues.left).toEqual([null, null, null]);
    expect(result.puzzle.clues.right).toEqual([3, null, null]);
  });
});
