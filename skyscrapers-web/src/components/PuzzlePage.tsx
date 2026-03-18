import { useReducer, useCallback, useEffect } from "react";
import type { Puzzle, GameState, GameAction, BoardCell } from "../types";
import { validateBoard } from "../validation";
import { PuzzleGrid } from "./PuzzleGrid";
import { NumberPad } from "./NumberPad";
import { GameControls } from "./GameControls";

function deepCopyBoard(board: BoardCell[][]): BoardCell[][] {
  return board.map((row) => row.map((cell) => ({ ...cell })));
}

function createInitialState(puzzle: Puzzle): GameState {
  return {
    puzzle,
    board: deepCopyBoard(puzzle.board),
    selectedCell: null,
    errors: new Set<string>(),
    completed: false,
  };
}

function checkCompleted(n: number, board: BoardCell[][], errors: Set<string>): boolean {
  if (errors.size > 0) return false;
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (board[r][c].value === null) return false;
    }
  }
  return true;
}

function gameReducer(state: GameState, action: GameAction): GameState {
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
      newBoard[r][c] = { value: action.value, given: false };
      const errors = validateBoard(n, newBoard);
      return {
        ...state,
        board: newBoard,
        errors,
        completed: checkCompleted(n, newBoard, errors),
      };
    }

    case "CLEAR_CELL": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      if (state.board[r][c].given) return state;
      const newBoard = deepCopyBoard(state.board);
      newBoard[r][c] = { value: null, given: false };
      const errors = validateBoard(n, newBoard);
      return {
        ...state,
        board: newBoard,
        errors,
        completed: false,
      };
    }

    case "RESET": {
      return createInitialState(puzzle);
    }

    case "CHECK": {
      const errors = validateBoard(n, state.board);
      return {
        ...state,
        errors,
        completed: checkCompleted(n, state.board, errors),
      };
    }

    default:
      return state;
  }
}

interface PuzzlePageProps {
  puzzle: Puzzle;
  onNewPuzzle: () => void;
}

export function PuzzlePage({ puzzle, onNewPuzzle }: PuzzlePageProps) {
  const [state, dispatch] = useReducer(gameReducer, puzzle, createInitialState);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const n = puzzle.n;
      const key = e.key;

      // Digits 1-n
      const digit = parseInt(key, 10);
      if (digit >= 1 && digit <= n) {
        dispatch({ type: "SET_VALUE", value: digit });
        return;
      }

      // Clear
      if (key === "0" || key === "Backspace" || key === "Delete") {
        e.preventDefault();
        dispatch({ type: "CLEAR_CELL" });
        return;
      }

      // Escape
      if (key === "Escape") {
        dispatch({ type: "DESELECT" });
        return;
      }

      // Arrow keys
      if (state.selectedCell !== null) {
        const [r, c] = state.selectedCell;
        let nr = r;
        let nc = c;

        if (key === "ArrowUp") nr = Math.max(0, r - 1);
        else if (key === "ArrowDown") nr = Math.min(n - 1, r + 1);
        else if (key === "ArrowLeft") nc = Math.max(0, c - 1);
        else if (key === "ArrowRight") nc = Math.min(n - 1, c + 1);
        else if (key === "Tab") {
          e.preventDefault();
          // Move to next empty cell
          for (let i = 1; i < n * n; i++) {
            const idx = (r * n + c + i) % (n * n);
            const tr = Math.floor(idx / n);
            const tc = idx % n;
            if (state.board[tr][tc].value === null && !state.board[tr][tc].given) {
              dispatch({ type: "SELECT_CELL", row: tr, col: tc });
              return;
            }
          }
          return;
        } else {
          return;
        }

        if (nr !== r || nc !== c) {
          e.preventDefault();
          dispatch({ type: "SELECT_CELL", row: nr, col: nc });
        }
      }
    },
    [puzzle.n, state.selectedCell, state.board]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const selectedValue =
    state.selectedCell !== null
      ? state.board[state.selectedCell[0]][state.selectedCell[1]].value
      : null;

  return (
    <div className="puzzle-page">
      <h1>Skyscrapers</h1>
      <PuzzleGrid
        puzzle={puzzle}
        board={state.board}
        selectedCell={state.selectedCell}
        errors={state.errors}
        onCellClick={(row, col) => dispatch({ type: "SELECT_CELL", row, col })}
      />
      <NumberPad
        n={puzzle.n}
        currentValue={selectedValue}
        onNumberSelect={(value) => dispatch({ type: "SET_VALUE", value })}
        onClear={() => dispatch({ type: "CLEAR_CELL" })}
      />
      <GameControls
        onReset={() => dispatch({ type: "RESET" })}
        onCheck={() => dispatch({ type: "CHECK" })}
        onNewPuzzle={onNewPuzzle}
        completed={state.completed}
      />
    </div>
  );
}
