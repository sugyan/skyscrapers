import { useReducer, useCallback, useEffect, useState } from "react";
import type {
  Puzzle,
  GameState,
  GameAction,
  BoardCell,
  HistorySnapshot,
} from "../types";
import type { Difficulty, HintResult } from "../wasm";
import { requestHint } from "../wasm";
import { relevantCells } from "../hint";
import { validateBoard } from "../validation";
import { PuzzleGrid } from "./PuzzleGrid";
import { NumberPad } from "./NumberPad";
import { GameControls } from "./GameControls";
import { HintPanel } from "./HintPanel";

function deepCopyBoard(board: BoardCell[][]): BoardCell[][] {
  return board.map((row) =>
    row.map((cell) => ({ ...cell, candidates: new Set(cell.candidates) })),
  );
}

const MAX_HISTORY = 100;

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
      const rowVals: Set<number>[] = Array.from({ length: n }, () => new Set());
      const colVals: Set<number>[] = Array.from({ length: n }, () => new Set());
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          const v = newBoard[r][c].value;
          if (v !== null) {
            rowVals[r].add(v);
            colVals[c].add(v);
          }
        }
      }
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

interface PuzzlePageProps {
  puzzle: Puzzle;
  solution: number[][];
  difficulty?: Difficulty | null;
  onNewPuzzle: () => void;
  onShowHowToPlay: () => void;
}

export function PuzzlePage({
  puzzle,
  solution,
  difficulty,
  onNewPuzzle,
  onShowHowToPlay,
}: PuzzlePageProps) {
  const [state, rawDispatch] = useReducer(
    gameReducer,
    { puzzle, solution },
    ({ puzzle, solution }) => createInitialState(puzzle, solution),
  );

  const [hint, setHint] = useState<HintResult | null>(null);
  const [hintError, setHintError] = useState<string | null>(null);
  const [filterValue, setFilterValue] = useState<number | null>(null);

  const dispatch = useCallback(
    (action: GameAction) => {
      // Any board-modifying action invalidates the current hint.
      switch (action.type) {
        case "SET_VALUE":
        case "CLEAR_CELL":
        case "TOGGLE_CANDIDATE":
        case "CLEAR_CANDIDATES":
        case "RESET":
        case "APPLY_HINT":
        case "FILL_ALL_CANDIDATES":
        case "UNDO":
          setHint(null);
          setHintError(null);
          break;
        default:
          break;
      }
      // Selecting a cell takes over the highlight, so clear any active filter.
      if (action.type === "SELECT_CELL") {
        setFilterValue(null);
      }
      rawDispatch(action);
    },
    [rawDispatch],
  );

  const handleHint = useCallback(async () => {
    // Pre-check: any user-placed value disagreeing with the solution makes
    // the solver's "next step" reasoning unsound — surface that first.
    const errors = new Set<string>();
    const n = puzzle.n;
    for (let r = 0; r < n; r++) {
      for (let c = 0; c < n; c++) {
        const v = state.board[r][c].value;
        if (v !== null && !state.board[r][c].given && v !== solution[r][c]) {
          errors.add(`${r},${c}`);
        }
      }
    }
    if (errors.size > 0) {
      rawDispatch({ type: "CHECK" });
      setHint(null);
      setHintError("Fix incorrect entries first.");
      return;
    }

    try {
      const result = await requestHint(puzzle, state.board);
      if (result === null) {
        setHint(null);
        setHintError("No hint available.");
      } else {
        setHint(result);
        setHintError(null);
        rawDispatch({ type: "DESELECT" });
      }
    } catch (e) {
      setHint(null);
      setHintError(`Hint failed: ${(e as Error).message}`);
    }
  }, [puzzle, solution, state.board]);

  const handleApplyHint = useCallback(() => {
    if (!hint) return;
    dispatch({
      type: "APPLY_HINT",
      actions: hint.step.actions,
      sync: {
        cells: relevantCells(hint),
        candidates: hint.candidates,
      },
    });
  }, [hint, dispatch]);

  const handleCloseHint = useCallback(() => {
    setHint(null);
    setHintError(null);
  }, []);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const n = puzzle.n;
      const key = e.key;

      // Undo: Ctrl/Cmd+Z. Only fire when there is something to undo, so a
      // no-op shortcut press doesn't clear the active hint via the dispatch
      // wrapper.
      if (
        (e.ctrlKey || e.metaKey) &&
        !e.shiftKey &&
        key.toLowerCase() === "z"
      ) {
        if (state.history.length === 0) return;
        e.preventDefault();
        dispatch({ type: "UNDO" });
        return;
      }

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
        if (state.selectedCell === null) {
          setFilterValue((prev) => (prev === digit ? null : digit));
        } else if (state.inputMode === "candidate") {
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
        if (state.selectedCell !== null) {
          dispatch({ type: "DESELECT" });
        } else {
          setFilterValue(null);
        }
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
    [
      puzzle.n,
      state.selectedCell,
      state.board,
      state.inputMode,
      state.history.length,
      dispatch,
    ],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const selectedCell =
    state.selectedCell !== null
      ? state.board[state.selectedCell[0]][state.selectedCell[1]]
      : null;

  const allEmptyHaveCandidates = (() => {
    const n = puzzle.n;
    for (let r = 0; r < n; r++) {
      for (let c = 0; c < n; c++) {
        const cell = state.board[r][c];
        if (cell.given) continue;
        if (cell.value !== null) continue;
        if (cell.candidates.size === 0) return false;
      }
    }
    return true;
  })();

  return (
    <div
      className="flex flex-col items-center p-5 sm:p-8"
      onClick={(e) => {
        // Tapping the outer container's empty area (not grid/numpad/etc.)
        // clears any current selection.
        if (e.target === e.currentTarget && state.selectedCell !== null) {
          dispatch({ type: "DESELECT" });
        }
      }}
    >
      <div className="flex items-center gap-3 mb-5 w-full max-w-md">
        <h1 className="text-2xl font-bold">Skyscrapers</h1>
        {difficulty && (
          <span className="text-xs px-2 py-0.5 rounded bg-slate-200 dark:bg-slate-700 capitalize">
            {difficulty}
          </span>
        )}
        <button
          onClick={onShowHowToPlay}
          className="w-7 h-7 rounded-full border border-gray-400 dark:border-slate-500 text-sm font-bold text-gray-500 dark:text-gray-400 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700"
          aria-label="How to Play"
        >
          ?
        </button>
        <button
          onClick={onNewPuzzle}
          className="ml-auto text-sm text-blue-600 dark:text-blue-400 underline cursor-pointer hover:text-blue-800 dark:hover:text-blue-300"
        >
          New puzzle
        </button>
      </div>
      {state.completed && (
        <p className="text-green-600 dark:text-green-400 font-bold text-xl mb-3 animate-bounce motion-reduce:animate-none">
          Completed!
        </p>
      )}
      <PuzzleGrid
        puzzle={puzzle}
        board={state.board}
        selectedCell={state.selectedCell}
        highlightValue={selectedCell?.value ?? filterValue}
        errors={state.errors}
        completed={state.completed}
        hint={hint}
        onCellClick={(row, col) => {
          if (
            state.selectedCell !== null &&
            state.selectedCell[0] === row &&
            state.selectedCell[1] === col
          ) {
            dispatch({ type: "DESELECT" });
          } else {
            dispatch({ type: "SELECT_CELL", row, col });
          }
        }}
      />
      <NumberPad
        n={puzzle.n}
        board={state.board}
        currentValue={selectedCell?.value ?? null}
        currentCandidates={selectedCell?.candidates ?? null}
        filterValue={selectedCell === null ? filterValue : null}
        filterMode={state.selectedCell === null}
        answerDisabled={selectedCell === null || selectedCell.given}
        memoDisabled={
          selectedCell === null ||
          selectedCell.given ||
          selectedCell.value !== null
        }
        onAnswer={(value) => dispatch({ type: "SET_VALUE", value })}
        onClearAnswer={() => dispatch({ type: "CLEAR_CELL" })}
        onToggleCandidate={(value) =>
          dispatch({ type: "TOGGLE_CANDIDATE", value })
        }
        onClearCandidates={() => dispatch({ type: "CLEAR_CANDIDATES" })}
        onFilter={(value) =>
          setFilterValue((prev) => (prev === value ? null : value))
        }
      />
      <GameControls
        canUndo={state.history.length > 0}
        canHint={allEmptyHaveCandidates}
        onUndo={() => dispatch({ type: "UNDO" })}
        onReset={() => dispatch({ type: "RESET" })}
        onHint={handleHint}
        onCheck={() => dispatch({ type: "CHECK" })}
        onFillCandidates={() => dispatch({ type: "FILL_ALL_CANDIDATES" })}
      />
      <HintPanel
        hint={hint}
        error={hintError}
        board={state.board}
        onApply={handleApplyHint}
        onClose={handleCloseHint}
      />
    </div>
  );
}
