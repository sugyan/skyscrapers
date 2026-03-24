import { describe, it, expect } from "vitest";
import { validateBoard } from "./validation";
import type { BoardCell } from "./types";

function makeBoard(n: number, values: (number | null)[][]): BoardCell[][] {
  return values.map((row) => row.map((v) => ({ value: v, given: v !== null })));
}

describe("validateBoard", () => {
  it("returns no errors for a valid complete board", () => {
    const board = makeBoard(3, [
      [1, 2, 3],
      [2, 3, 1],
      [3, 1, 2],
    ]);
    expect(validateBoard(3, board).size).toBe(0);
  });

  it("returns no errors for a partial board with no conflicts", () => {
    const board = makeBoard(3, [
      [1, null, null],
      [null, 2, null],
      [null, null, 3],
    ]);
    expect(validateBoard(3, board).size).toBe(0);
  });

  it("detects row duplicates", () => {
    const board = makeBoard(3, [
      [1, 1, null],
      [null, null, null],
      [null, null, null],
    ]);
    const errors = validateBoard(3, board);
    expect(errors.has("0,0")).toBe(true);
    expect(errors.has("0,1")).toBe(true);
    expect(errors.size).toBe(2);
  });

  it("detects column duplicates", () => {
    const board = makeBoard(3, [
      [2, null, null],
      [null, null, null],
      [2, null, null],
    ]);
    const errors = validateBoard(3, board);
    expect(errors.has("0,0")).toBe(true);
    expect(errors.has("2,0")).toBe(true);
    expect(errors.size).toBe(2);
  });

  it("detects out-of-range values", () => {
    const board = makeBoard(3, [
      [0 as unknown as number, null, null],
      [null, 4, null],
      [null, null, null],
    ]);
    // Manually set value 0 (bypassing makeBoard's null conversion)
    board[0][0] = { value: 0, given: false };
    const errors = validateBoard(3, board);
    expect(errors.has("0,0")).toBe(true);
    expect(errors.has("1,1")).toBe(true);
  });
});
