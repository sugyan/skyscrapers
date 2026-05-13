import type {
  BoardCell,
  GameAction,
  GameState,
  HistorySnapshot,
  Puzzle,
} from "./types";
import { validateBoard } from "../utils/validation";
import { computeRowColValues } from "../utils/board";

const MAX_HISTORY = 100;

function deepCopyBoard(board: BoardCell[][]): BoardCell[][] {
  return board.map((row) =>
    row.map((cell) => ({ ...cell, candidates: new Set(cell.candidates) })),
  );
}

function pushHistory(state: GameState): HistorySnapshot[] {
  const next = [
    ...state.history,
    {
      board: state.board,
      errors: state.errors,
      completed: state.completed,
    },
  ];
  if (next.length > MAX_HISTORY) next.shift();
  return next;
}

export function createInitialState(
  puzzle: Puzzle,
  solution: number[][],
): GameState {
  const board = puzzle.board.map((row) =>
    row.map((cell) => ({ ...cell, candidates: new Set<number>() })),
  );
  return {
    puzzle,
    solution,
    board,
    selectedCell: null,
    errors: new Set<string>(),
    completed: false,
    inputMode: "answer",
    history: [],
  };
}

function checkCompleted(
  n: number,
  board: BoardCell[][],
  errors: Set<string>,
): boolean {
  if (errors.size > 0) return false;
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (board[r][c].value === null) return false;
    }
  }
  return true;
}

export function gameReducer(state: GameState, action: GameAction): GameState {
  const { puzzle } = state;
  const n = puzzle.n;

  switch (action.type) {
    case "SELECT_CELL": {
      const { row, col } = action;
      return { ...state, selectedCell: [row, col] };
    }

    case "DESELECT": {
      return { ...state, selectedCell: null };
    }

    case "SET_VALUE": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      if (state.board[r][c].given) return state;
      const newBoard = deepCopyBoard(state.board);
      newBoard[r][c] = {
        value: action.value,
        given: false,
        candidates: new Set(),
      };
      const errors = validateBoard(n, newBoard);
      return {
        ...state,
        board: newBoard,
        errors,
        completed: checkCompleted(n, newBoard, errors),
        history: pushHistory(state),
      };
    }

    case "CLEAR_CELL": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      if (state.board[r][c].given) return state;
      const newBoard = deepCopyBoard(state.board);
      newBoard[r][c] = { value: null, given: false, candidates: new Set() };
      const errors = validateBoard(n, newBoard);
      return {
        ...state,
        board: newBoard,
        errors,
        completed: false,
        history: pushHistory(state),
      };
    }

    case "TOGGLE_CANDIDATE": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      const cell = state.board[r][c];
      if (cell.given || cell.value !== null) return state;
      const newBoard = deepCopyBoard(state.board);
      const candidates = newBoard[r][c].candidates;
      if (candidates.has(action.value)) {
        candidates.delete(action.value);
      } else {
        candidates.add(action.value);
      }
      return { ...state, board: newBoard, history: pushHistory(state) };
    }

    case "CLEAR_CANDIDATES": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      if (state.board[r][c].given) return state;
      if (state.board[r][c].candidates.size === 0) return state;
      const newBoard = deepCopyBoard(state.board);
      newBoard[r][c].candidates = new Set();
      return { ...state, board: newBoard, history: pushHistory(state) };
    }

    case "SET_INPUT_MODE": {
      return { ...state, inputMode: action.mode };
    }

    case "RESET": {
      return createInitialState(puzzle, state.solution);
    }

    case "CHECK": {
      const errors = new Set<string>();
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          const v = state.board[r][c].value;
          if (v !== null && v !== state.solution[r][c]) {
            errors.add(`${r},${c}`);
          }
        }
      }
      return {
        ...state,
        errors,
        completed: checkCompleted(n, state.board, errors),
      };
    }

    case "UNDO": {
      if (state.history.length === 0) return state;
      const last = state.history[state.history.length - 1];
      return {
        ...state,
        board: last.board,
        errors: last.errors,
        completed: last.completed,
        history: state.history.slice(0, -1),
      };
    }

    case "APPLY_HINT": {
      const newBoard = deepCopyBoard(state.board);
      if (action.sync) {
        for (const [r, c] of action.sync.cells) {
          const cell = newBoard[r][c];
          if (cell.given || cell.value !== null) continue;
          cell.candidates = new Set(action.sync.candidates[r][c]);
        }
      }
      for (const a of action.actions) {
        if (newBoard[a.row][a.col].given) continue;
        if (a.kind === "place") {
          newBoard[a.row][a.col] = {
            value: a.value,
            given: false,
            candidates: new Set(),
          };
        } else {
          if (newBoard[a.row][a.col].value !== null) continue;
          newBoard[a.row][a.col].candidates.delete(a.value);
        }
      }
      const errors = validateBoard(n, newBoard);
      return {
        ...state,
        board: newBoard,
        errors,
        completed: checkCompleted(n, newBoard, errors),
        history: pushHistory(state),
      };
    }

    case "FILL_ALL_CANDIDATES": {
      const newBoard = deepCopyBoard(state.board);
      const { rowVals, colVals } = computeRowColValues(newBoard);
      let changed = false;
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          const cell = newBoard[r][c];
          if (cell.given) continue;
          if (cell.value !== null) continue;
          if (cell.candidates.size > 0) continue;
          const candidates = new Set<number>();
          for (let v = 1; v <= n; v++) {
            if (!rowVals[r].has(v) && !colVals[c].has(v)) candidates.add(v);
          }
          if (candidates.size === 0) continue;
          cell.candidates = candidates;
          changed = true;
        }
      }
      if (!changed) return state;
      return { ...state, board: newBoard, history: pushHistory(state) };
    }

    default:
      return state;
  }
}
