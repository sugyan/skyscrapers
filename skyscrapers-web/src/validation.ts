import type { BoardCell } from "./types";

export function validateBoard(n: number, board: BoardCell[][]): Set<string> {
  const errors = new Set<string>();

  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      const v = board[r][c].value;
      if (v !== null && (v < 1 || v > n)) {
        errors.add(`${r},${c}`);
      }
    }
  }

  // Row uniqueness
  for (let r = 0; r < n; r++) {
    const seen = new Map<number, number[]>();
    for (let c = 0; c < n; c++) {
      const v = board[r][c].value;
      if (v !== null) {
        if (!seen.has(v)) {
          seen.set(v, []);
        }
        seen.get(v)!.push(c);
      }
    }
    for (const cols of seen.values()) {
      if (cols.length > 1) {
        for (const c of cols) {
          errors.add(`${r},${c}`);
        }
      }
    }
  }

  // Column uniqueness
  for (let c = 0; c < n; c++) {
    const seen = new Map<number, number[]>();
    for (let r = 0; r < n; r++) {
      const v = board[r][c].value;
      if (v !== null) {
        if (!seen.has(v)) {
          seen.set(v, []);
        }
        seen.get(v)!.push(r);
      }
    }
    for (const rows of seen.values()) {
      if (rows.length > 1) {
        for (const r of rows) {
          errors.add(`${r},${c}`);
        }
      }
    }
  }

  return errors;
}
