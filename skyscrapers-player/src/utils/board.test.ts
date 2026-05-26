import { describe, it, expect } from "vitest";
import { withAllCandidatesFilled } from "./board";
import type { BoardCell } from "../state/types";

function makeBoard(
  cells: ({ v?: number; given?: boolean; cands?: number[] } | null)[][],
): BoardCell[][] {
  return cells.map((row) =>
    row.map((c) => {
      if (c === null) {
        return { value: null, given: false, candidates: new Set<number>() };
      }
      return {
        value: c.v ?? null,
        given: c.given ?? false,
        candidates: new Set<number>(c.cands ?? []),
      };
    }),
  );
}

describe("withAllCandidatesFilled", () => {
  it("fills candidates for empty cells with no marks", () => {
    const board = makeBoard([
      [{ v: 1, given: true }, null, null],
      [null, null, null],
      [null, null, { v: 3, given: true }],
    ]);
    const next = withAllCandidatesFilled(board);
    // Row 0 has 1; col 1 has nothing → cell (0,1) candidates = {2,3}
    expect([...next[0][1].candidates].sort((a, b) => a - b)).toEqual([2, 3]);
    // Cell (1,1): row 1 empty, col 1 empty → {1,2,3}
    expect([...next[1][1].candidates].sort((a, b) => a - b)).toEqual([1, 2, 3]);
    // Cell (2,0): col 0 has 1; row 2 has 3 → {2}
    expect([...next[2][0].candidates].sort((a, b) => a - b)).toEqual([2]);
  });

  it("returns the same reference when nothing changes", () => {
    // All non-given empty cells already have at least one candidate marked.
    const board = makeBoard([
      [{ v: 1, given: true }, { cands: [2] }, { cands: [3] }],
      [{ cands: [2] }, { cands: [1] }, { cands: [1] }],
      [{ cands: [2] }, { cands: [3] }, { v: 3, given: true }],
    ]);
    expect(withAllCandidatesFilled(board)).toBe(board);
  });

  it("leaves cells with existing candidates untouched (no overwrite)", () => {
    const board = makeBoard([
      [{ v: 1, given: true }, { cands: [2] }, null],
      [null, null, null],
      [null, null, { v: 3, given: true }],
    ]);
    const next = withAllCandidatesFilled(board);
    // The pre-marked cell at (0,1) keeps just its single candidate.
    expect([...next[0][1].candidates]).toEqual([2]);
    // But (0,2), which had none, gets filled.
    expect(next[0][2].candidates.size).toBeGreaterThan(0);
  });

  it("skips given cells and filled non-given cells", () => {
    const board = makeBoard([
      [{ v: 1, given: true }, null, null],
      [{ v: 2, given: false }, null, null],
      [null, null, null],
    ]);
    const next = withAllCandidatesFilled(board);
    // Given cell unchanged.
    expect(next[0][0].value).toBe(1);
    expect(next[0][0].candidates.size).toBe(0);
    // User-placed value unchanged; no candidates filled in.
    expect(next[1][0].value).toBe(2);
    expect(next[1][0].candidates.size).toBe(0);
  });

  it("reuses unchanged cells by reference (structural sharing)", () => {
    const board = makeBoard([
      [{ v: 1, given: true }, null, null],
      [null, null, null],
      [null, null, { v: 3, given: true }],
    ]);
    const next = withAllCandidatesFilled(board);
    expect(next).not.toBe(board);
    // The given cells are not modified and should be the same references.
    expect(next[0][0]).toBe(board[0][0]);
    expect(next[2][2]).toBe(board[2][2]);
    // Cells that got candidates filled in are fresh objects.
    expect(next[0][1]).not.toBe(board[0][1]);
  });

  it("does not mutate the input board", () => {
    const board = makeBoard([
      [{ v: 1, given: true }, null, null],
      [null, null, null],
      [null, null, { v: 3, given: true }],
    ]);
    const snapshot = board.map((row) =>
      row.map((c) => ({ ...c, candidates: new Set(c.candidates) })),
    );
    withAllCandidatesFilled(board);
    for (let r = 0; r < board.length; r++) {
      for (let c = 0; c < board[r].length; c++) {
        expect(board[r][c].value).toBe(snapshot[r][c].value);
        expect(board[r][c].given).toBe(snapshot[r][c].given);
        expect([...board[r][c].candidates]).toEqual([
          ...snapshot[r][c].candidates,
        ]);
      }
    }
  });
});
