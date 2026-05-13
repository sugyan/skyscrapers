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
