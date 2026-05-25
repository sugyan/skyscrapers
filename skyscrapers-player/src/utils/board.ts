import type { BoardCell } from "../state/types";

export function computeRowColValues(board: BoardCell[][]): {
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

/**
 * Returns a new board where every empty, non-given cell with no pencil marks
 * has its candidate set populated to all values not present in the cell's row
 * or column. Cells that already have any candidates are left untouched.
 *
 * If nothing changes, the original `board` reference is returned. Otherwise a
 * fresh outer array is returned, but unchanged cells are reused by reference
 * (structural sharing) — callers should treat the board as immutable and
 * never mutate cells in place.
 */
export function withAllCandidatesFilled(board: BoardCell[][]): BoardCell[][] {
  const n = board.length;
  const { rowVals, colVals } = computeRowColValues(board);
  let changed = false;
  const next: BoardCell[][] = board.map((row, r) =>
    row.map((cell, c) => {
      if (cell.given) return cell;
      if (cell.value !== null) return cell;
      if (cell.candidates.size > 0) return cell;
      const candidates = new Set<number>();
      for (let v = 1; v <= n; v++) {
        if (!rowVals[r].has(v) && !colVals[c].has(v)) candidates.add(v);
      }
      if (candidates.size === 0) return cell;
      changed = true;
      return { ...cell, candidates };
    }),
  );
  return changed ? next : board;
}

export function blockedValuesAt(
  board: BoardCell[][],
  row: number,
  col: number,
): Set<number> {
  const n = board.length;
  const blocked = new Set<number>();
  for (let i = 0; i < n; i++) {
    const rv = board[row][i].value;
    if (rv !== null) blocked.add(rv);
    const cv = board[i][col].value;
    if (cv !== null) blocked.add(cv);
  }
  return blocked;
}
