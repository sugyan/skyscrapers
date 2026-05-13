import { useReducer, useCallback, useEffect, useRef, useState } from "react";
import type { Puzzle, GameAction } from "../state/types";
import type {
  Difficulty,
  HintResult,
  SkyscrapersEngine,
} from "../engine/types";
import { createInitialState, gameReducer } from "../state/reducer";
import { relevantCells } from "../utils/hint";
import { blockedValuesAt } from "../utils/board";
import { PuzzleGrid } from "./PuzzleGrid";
import { NumberPad } from "./NumberPad";
import { GameControls } from "./GameControls";
import { HintPanel } from "./HintPanel";
import { ConfirmDialog } from "./ConfirmDialog";

const DOUBLE_TAP_MS = 350;

export interface PlayerProps {
  puzzle: Puzzle;
  solution: number[][];
  engine: SkyscrapersEngine;
  difficulty?: Difficulty | null;
  onNewPuzzle?: () => void;
  onShowHowToPlay?: () => void;
}

export function Player({
  puzzle,
  solution,
  engine,
  difficulty,
  onNewPuzzle,
  onShowHowToPlay,
}: PlayerProps) {
  const [state, rawDispatch] = useReducer(
    gameReducer,
    { puzzle, solution },
    ({ puzzle, solution }) => createInitialState(puzzle, solution),
  );

  const [hint, setHint] = useState<HintResult | null>(null);
  const [hintError, setHintError] = useState<string | null>(null);
  const [filterValue, setFilterValue] = useState<number | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const lastTapRef = useRef<{ r: number; c: number; t: number } | null>(null);

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
      const result = await engine.requestHint(puzzle, state.board);
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
  }, [engine, puzzle, solution, state.board]);

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
        {onShowHowToPlay && (
          <button
            onClick={onShowHowToPlay}
            className="w-7 h-7 rounded-full border border-gray-400 dark:border-slate-500 text-sm font-bold text-gray-500 dark:text-gray-400 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700"
            aria-label="How to Play"
          >
            ?
          </button>
        )}
        {onNewPuzzle && (
          <button
            onClick={onNewPuzzle}
            className="ml-auto text-sm text-blue-600 dark:text-blue-400 underline cursor-pointer hover:text-blue-800 dark:hover:text-blue-300"
          >
            New puzzle
          </button>
        )}
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
          const cell = state.board[row][col];
          const now = Date.now();
          const prev = lastTapRef.current;
          const isDouble =
            prev !== null &&
            prev.r === row &&
            prev.c === col &&
            now - prev.t <= DOUBLE_TAP_MS;

          if (
            isDouble &&
            !cell.given &&
            cell.value === null &&
            cell.candidates.size > 0
          ) {
            const blocked = blockedValuesAt(state.board, row, col);
            const effective: number[] = [];
            for (const v of cell.candidates) {
              if (!blocked.has(v)) effective.push(v);
            }
            if (effective.length === 1) {
              const only = effective[0];
              if (
                state.selectedCell === null ||
                state.selectedCell[0] !== row ||
                state.selectedCell[1] !== col
              ) {
                dispatch({ type: "SELECT_CELL", row, col });
              }
              dispatch({ type: "SET_VALUE", value: only });
              lastTapRef.current = null;
              return;
            }
          }

          lastTapRef.current = { r: row, c: col, t: now };

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
        onReset={() => setConfirmReset(true)}
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
      {confirmReset && (
        <ConfirmDialog
          title="Reset puzzle?"
          message="All entries and memo marks will be cleared. This cannot be undone."
          confirmLabel="Reset"
          destructive
          onConfirm={() => {
            dispatch({ type: "RESET" });
            setConfirmReset(false);
          }}
          onCancel={() => setConfirmReset(false)}
        />
      )}
    </div>
  );
}
