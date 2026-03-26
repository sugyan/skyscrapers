import type { Puzzle, BoardCell, ClueValue } from "./types";

export function decodePuzzle(encoded: string): Puzzle {
  if (!/^\d+$/.test(encoded)) {
    throw new Error("Encoded string must contain only digits");
  }

  const n = parseInt(encoded[0], 10);
  if (n < 5 || n > 9) {
    throw new Error(`Invalid board size: ${n}. Must be 5–9`);
  }

  const expectedLength = 1 + 4 * n + n * n;
  if (encoded.length !== expectedLength) {
    throw new Error(
      `Invalid length: expected ${expectedLength}, got ${encoded.length}`,
    );
  }

  let pos = 1;

  const readClues = (): ClueValue[] => {
    const clues: ClueValue[] = [];
    for (let i = 0; i < n; i++) {
      const v = parseInt(encoded[pos++], 10);
      if (v < 0 || v > n) {
        throw new Error(`Invalid clue value: ${v}. Must be 0–${n}`);
      }
      clues.push(v === 0 ? null : v);
    }
    return clues;
  };

  const top = readClues();
  const bottom = readClues();
  const left = readClues();
  const right = readClues();

  const board: BoardCell[][] = [];
  for (let r = 0; r < n; r++) {
    const row: BoardCell[] = [];
    for (let c = 0; c < n; c++) {
      const v = parseInt(encoded[pos++], 10);
      if (v < 0 || v > n) {
        throw new Error(`Invalid cell value: ${v}. Must be 0–${n}`);
      }
      row.push({
        value: v === 0 ? null : v,
        given: v !== 0,
        candidates: new Set(),
      });
    }
    board.push(row);
  }

  return { n, board, clues: { top, bottom, left, right } };
}

export function encodePuzzle(puzzle: Puzzle): string {
  const { n, board, clues } = puzzle;
  let result = String(n);

  const encodeClues = (arr: ClueValue[]) => {
    for (const v of arr) {
      result += String(v ?? 0);
    }
  };

  encodeClues(clues.top);
  encodeClues(clues.bottom);
  encodeClues(clues.left);
  encodeClues(clues.right);

  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      result += String(board[r][c].value ?? 0);
    }
  }

  return result;
}
