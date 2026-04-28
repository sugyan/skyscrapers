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
  candidates: Set<number>;
}

export type InputMode = "answer" | "candidate";

export interface GameState {
  puzzle: Puzzle;
  solution: number[][];
  board: BoardCell[][];
  selectedCell: [number, number] | null;
  errors: Set<string>;
  completed: boolean;
  inputMode: InputMode;
}

export type GameAction =
  | { type: "SELECT_CELL"; row: number; col: number }
  | { type: "DESELECT" }
  | { type: "SET_VALUE"; value: number }
  | { type: "CLEAR_CELL" }
  | { type: "TOGGLE_CANDIDATE"; value: number }
  | { type: "CLEAR_CANDIDATES" }
  | { type: "SET_INPUT_MODE"; mode: InputMode }
  | { type: "RESET" }
  | { type: "CHECK" }
  | {
      type: "APPLY_HINT";
      actions: ReadonlyArray<
        | { kind: "place"; row: number; col: number; value: number }
        | { kind: "eliminate"; row: number; col: number; value: number }
      >;
    }
  | {
      type: "SYNC_CANDIDATES";
      cells: ReadonlyArray<[number, number]>;
      candidates: number[][][];
    };
