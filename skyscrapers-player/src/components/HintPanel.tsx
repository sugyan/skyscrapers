import type { BoardCell } from "../state/types";
import type { HintResult } from "../engine/types";
import {
  TECHNIQUE_DESCRIPTIONS,
  TECHNIQUE_LABELS,
  candidateDiffs,
  cellLabel,
  reasonText,
  type CellCandidateDiff,
} from "../utils/hint";

interface HintPanelProps {
  hint: HintResult | null;
  error: string | null;
  autoFilledMemo?: boolean;
  board: BoardCell[][];
  onApply: () => void;
  onClose: () => void;
}

function actionSummary(hint: HintResult): string {
  const parts = hint.step.actions.map((a) => {
    const target = cellLabel(a.row, a.col);
    return a.kind === "place"
      ? `Place ${a.value} at ${target}`
      : `Eliminate ${a.value} from ${target}`;
  });
  return parts.join(", ");
}

function CandidateChips({ diff, n }: { diff: CellCandidateDiff; n: number }) {
  const expectedSet = new Set(diff.expected);
  const userSet = new Set([
    ...diff.extra,
    ...diff.expected.filter((v) => !diff.missing.includes(v)),
  ]);
  const all: number[] = [];
  for (let v = 1; v <= n; v++) all.push(v);

  return (
    <div className="flex flex-wrap gap-1">
      {all.map((v) => {
        const inExpected = expectedSet.has(v);
        const inUser = userSet.has(v);
        let cls =
          "inline-flex items-center justify-center w-6 h-6 text-xs rounded border ";
        if (inExpected && inUser) {
          cls +=
            "bg-green-100 text-green-800 border-green-400 dark:bg-green-900/40 dark:text-green-200 dark:border-green-700";
        } else if (inUser && !inExpected) {
          cls +=
            "bg-red-100 text-red-800 border-red-400 dark:bg-red-900/40 dark:text-red-200 dark:border-red-700";
        } else if (inExpected && !inUser) {
          cls +=
            "bg-transparent text-gray-400 border-dashed border-gray-400 dark:text-slate-500 dark:border-slate-500";
        } else {
          return null;
        }
        return (
          <span key={v} className={cls}>
            {v}
          </span>
        );
      })}
    </div>
  );
}

export function HintPanel({
  hint,
  error,
  autoFilledMemo,
  board,
  onApply,
  onClose,
}: HintPanelProps) {
  if (!hint && !error) return null;

  const btnClass =
    "px-3 py-1 text-sm border border-gray-400 dark:border-slate-600 rounded bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700";
  const primaryBtn =
    "px-3 py-1 text-sm border border-blue-500 dark:border-blue-400 rounded bg-blue-500 dark:bg-blue-600 text-white cursor-pointer hover:bg-blue-600 dark:hover:bg-blue-700";

  if (error) {
    return (
      <div className="w-full max-w-md mt-3 p-3 border border-amber-400 bg-amber-50 dark:bg-amber-900/30 dark:border-amber-700 rounded">
        <div className="flex items-start justify-between gap-3">
          <p className="text-sm text-amber-800 dark:text-amber-200">{error}</p>
          <button onClick={onClose} className={btnClass} aria-label="Close">
            ✕
          </button>
        </div>
      </div>
    );
  }

  if (!hint) return null;

  const n = board.length;
  const diffs = candidateDiffs(hint, board);
  // Apply runs a candidate sync before executing the step's actions, so
  // "candidate not currently marked" is no longer a reliable no-op signal —
  // the sync may bring the value back. An eliminate is only a guaranteed
  // no-op when the cell is already confirmed.
  const isActionNoOp = (a: HintResult["step"]["actions"][number]): boolean => {
    const cell = board[a.row][a.col];
    if (a.kind === "place") {
      return cell.value === a.value;
    }
    return cell.value !== null;
  };
  const allNoOp = hint.step.actions.every(isActionNoOp);

  return (
    <div className="w-full max-w-md mt-3 p-3 border border-amber-400 bg-amber-50 dark:bg-amber-900/30 dark:border-amber-700 rounded text-sm">
      <div className="flex items-start justify-between gap-3 mb-2">
        <div>
          <div className="font-semibold text-amber-900 dark:text-amber-200">
            {TECHNIQUE_LABELS[hint.step.technique]}
          </div>
          <div className="text-xs italic text-amber-700/80 dark:text-amber-300/70 mb-1">
            {TECHNIQUE_DESCRIPTIONS[hint.step.technique]}
          </div>
          <div className="text-amber-800 dark:text-amber-100">
            {actionSummary(hint)}
          </div>
        </div>
        <button onClick={onClose} className={btnClass} aria-label="Close">
          ✕
        </button>
      </div>

      <p className="text-amber-700 dark:text-amber-200/90 mb-3">
        {reasonText(hint)}
      </p>

      {autoFilledMemo && (
        <p className="text-xs text-amber-700/90 dark:text-amber-300/80 mb-3">
          Memo marks were filled in automatically so the hint could be computed.
        </p>
      )}

      {diffs.length > 0 && (
        <div className="mb-3">
          <div className="text-xs uppercase tracking-wide text-amber-700 dark:text-amber-300/80 mb-1">
            Expected candidates
          </div>
          <div className="space-y-1">
            {diffs.map((d) => (
              <div
                key={`${d.row},${d.col}`}
                className="flex items-center gap-2"
              >
                <span className="font-mono text-xs w-12 text-amber-900 dark:text-amber-100">
                  {cellLabel(d.row, d.col)}
                </span>
                {d.confirmed ? (
                  <span className="text-xs text-amber-700 dark:text-amber-300/80">
                    confirmed = {board[d.row][d.col].value}
                  </span>
                ) : d.userEmpty ? (
                  <>
                    <CandidateChips diff={d} n={n} />
                    <span className="text-xs text-amber-700 dark:text-amber-300/80">
                      (no marks yet)
                    </span>
                  </>
                ) : (
                  <CandidateChips diff={d} n={n} />
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex gap-2 justify-end">
        <button
          onClick={onApply}
          disabled={allNoOp}
          className={
            allNoOp
              ? "px-3 py-1 text-sm border border-gray-300 dark:border-slate-700 rounded bg-gray-100 dark:bg-slate-800 text-gray-400 dark:text-slate-500 cursor-not-allowed"
              : primaryBtn
          }
        >
          Apply
        </button>
      </div>
    </div>
  );
}
