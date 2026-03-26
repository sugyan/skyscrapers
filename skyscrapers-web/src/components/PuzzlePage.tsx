import { useReducer, useCallback, useEffect } from "react";
import type { Puzzle, GameState, GameAction, BoardCell } from "../types";
import { validateBoard } from "../validation";
import { PuzzleGrid } from "./PuzzleGrid";
import { NumberPad } from "./NumberPad";
import { GameControls } from "./GameControls";

function deepCopyBoard(board: BoardCell[][]): BoardCell[][] {
  return board.map((row) =>
    row.map((cell) => ({ ...cell, candidates: new Set(cell.candidates) })),
  );
}

function createInitialState(puzzle: Puzzle, solution: number[][]): GameState {
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
      return { ...state, board: newBoard };
    }

    case "CLEAR_CANDIDATES": {
      if (state.selectedCell === null) return state;
      const [r, c] = state.selectedCell;
      if (state.board[r][c].given) return state;
      const newBoard = deepCopyBoard(state.board);
      newBoard[r][c].candidates = new Set();
      return { ...state, board: newBoard };
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

    default:
      return state;
  }
}

interface PuzzlePageProps {
  puzzle: Puzzle;
  solution: number[][];
  onNewPuzzle: () => void;
}

export function PuzzlePage({ puzzle, solution, onNewPuzzle }: PuzzlePageProps) {
  const [state, dispatch] = useReducer(
    gameReducer,
    { puzzle, solution },
    ({ puzzle, solution }) => createInitialState(puzzle, solution),
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const n = puzzle.n;
      const key = e.key;

      // Space toggles input mode
      if (key === " ") {
        e.preventDefault();
        dispatch({
          type: "SET_INPUT_MODE",
          mode: state.inputMode === "answer" ? "candidate" : "answer",
        });
        return;
      }

      // Digits 1-n
      const digit = parseInt(key, 10);
      if (digit >= 1 && digit <= n) {
        if (state.inputMode === "candidate") {
          dispatch({ type: "TOGGLE_CANDIDATE", value: digit });
        } else {
          dispatch({ type: "SET_VALUE", value: digit });
        }
        return;
      }

      // Clear
      if (key === "0" || key === "Backspace" || key === "Delete") {
        if (
          state.selectedCell !== null &&
          !state.board[state.selectedCell[0]][state.selectedCell[1]].given
        ) {
          e.preventDefault();
          if (state.inputMode === "candidate") {
            dispatch({ type: "CLEAR_CANDIDATES" });
          } else {
            dispatch({ type: "CLEAR_CELL" });
          }
        }
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
            if (
              state.board[tr][tc].value === null &&
              !state.board[tr][tc].given
            ) {
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
    [puzzle.n, state.selectedCell, state.board, state.inputMode],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const selectedCell =
    state.selectedCell !== null
      ? state.board[state.selectedCell[0]][state.selectedCell[1]]
      : null;

  return (
    <div className="flex flex-col items-center p-5 sm:p-8">
      <h1 className="text-2xl font-bold mb-5">Skyscrapers</h1>
      <PuzzleGrid
        puzzle={puzzle}
        board={state.board}
        selectedCell={state.selectedCell}
        errors={state.errors}
        completed={state.completed}
        onCellClick={(row, col) => dispatch({ type: "SELECT_CELL", row, col })}
      />
      <NumberPad
        n={puzzle.n}
        inputMode={state.inputMode}
        currentValue={selectedCell?.value ?? null}
        currentCandidates={selectedCell?.candidates ?? null}
        onNumberSelect={(value) => {
          if (state.inputMode === "candidate") {
            dispatch({ type: "TOGGLE_CANDIDATE", value });
          } else {
            dispatch({ type: "SET_VALUE", value });
          }
        }}
        onClear={() => {
          if (state.inputMode === "candidate") {
            dispatch({ type: "CLEAR_CANDIDATES" });
          } else {
            dispatch({ type: "CLEAR_CELL" });
          }
        }}
        onModeChange={(mode) => dispatch({ type: "SET_INPUT_MODE", mode })}
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
