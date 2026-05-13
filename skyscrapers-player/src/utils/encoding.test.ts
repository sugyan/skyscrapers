import { describe, it, expect } from "vitest";
import { decodePuzzle, encodePuzzle } from "./encoding";

describe("decodePuzzle", () => {
  it("decodes a 5×5 puzzle", () => {
    const encoded = "5003000100303000003420000000000000000000000020";
    const puzzle = decodePuzzle(encoded);

    expect(puzzle.n).toBe(5);
    // encoded clues: top=00300, bottom=01003, left=03000, right=00342
    expect(puzzle.clues.top).toEqual([null, null, 3, null, null]);
    expect(puzzle.clues.bottom).toEqual([null, 1, null, null, 3]);
    expect(puzzle.clues.left).toEqual([null, 3, null, null, null]);
    expect(puzzle.clues.right).toEqual([null, null, 3, 4, 2]);
    // board cells: 0 → null (not given), nonzero → given
    expect(puzzle.board[0][0].value).toBeNull();
    // board[4][3] = 2 (given), board[4][4] = null
    expect(puzzle.board[4][3].value).toBe(2);
    expect(puzzle.board[4][3].given).toBe(true);
    expect(puzzle.board[4][4].value).toBeNull();
  });

  it("throws on non-digit input", () => {
    expect(() => decodePuzzle("abc")).toThrow("only digits");
  });

  it("throws on invalid board size", () => {
    expect(() => decodePuzzle("3")).toThrow("Invalid board size");
  });

  it("throws on wrong length", () => {
    expect(() => decodePuzzle("51234")).toThrow("Invalid length");
  });
});

describe("encodePuzzle", () => {
  it("round-trips a 5×5 puzzle", () => {
    const encoded = "5003000100303000003420000000000000000000000020";
    const puzzle = decodePuzzle(encoded);
    expect(encodePuzzle(puzzle)).toBe(encoded);
  });

  it("round-trips a 7×7 puzzle", () => {
    const encoded =
      "703023204052100000000533034000000030000004000000000020000000000040000000000000";
    const puzzle = decodePuzzle(encoded);
    expect(encodePuzzle(puzzle)).toBe(encoded);
  });
});
