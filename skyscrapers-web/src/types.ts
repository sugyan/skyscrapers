export type ClueValue = number | null;

export interface Puzzle {
  n: number;
  board: BoardCell[][];
  clues: {
    top: ClueValue[];
    bottom: ClueValue[];
    left: ClueValue[];
    right: ClueValue[];
  };
}

export interface BoardCell {
  value: number | null;
  given: boolean;
}

export interface GameState {
  puzzle: Puzzle;
  board: BoardCell[][];
  selectedCell: [number, number] | null;
  errors: Set<string>;
  completed: boolean;
}

export type GameAction =
  | { type: "SELECT_CELL"; row: number; col: number }
  | { type: "DESELECT" }
  | { type: "SET_VALUE"; value: number }
  | { type: "CLEAR_CELL" }
  | { type: "RESET" }
  | { type: "CHECK" };
