import type { BoardCell } from "../types";

interface NumberPadProps {
  n: number;
  board: BoardCell[][];
  currentValue: number | null;
  currentCandidates: Set<number> | null;
  memoDisabled: boolean;
  onAnswer: (value: number) => void;
  onClearAnswer: () => void;
  onToggleCandidate: (value: number) => void;
  onClearCandidates: () => void;
}

function RemainingBars({ remaining }: { remaining: number }) {
  if (remaining <= 0) return null;
  return (
    <div className="flex flex-col items-center gap-0.5 mb-1.5">
      {Array.from({ length: remaining }, (_, i) => (
        <div
          key={i}
          className="w-4 h-0.5 bg-blue-400 dark:bg-blue-500 rounded-full"
        />
      ))}
    </div>
  );
}

function EraserIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="w-5 h-5"
    >
      <path d="M20 20H7L3 16c-.8-.8-.8-2 0-2.8L14.6 1.6c.8-.8 2-.8 2.8 0L21 5.2c.8.8.8 2 0 2.8L12 17" />
      <path d="M6 11l4 4" />
    </svg>
  );
}

export function NumberPad({
  n,
  board,
  currentValue,
  currentCandidates,
  memoDisabled,
  onAnswer,
  onClearAnswer,
  onToggleCandidate,
  onClearCandidates,
}: NumberPadProps) {
  // Count how many of each digit are placed on the board
  const placedCounts = new Map<number, number>();
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      const v = board[r][c].value;
      if (v !== null) {
        placedCounts.set(v, (placedCounts.get(v) ?? 0) + 1);
      }
    }
  }

  const btnSize = "w-12 h-12";
  const btnBase =
    `${btnSize} text-lg border rounded-md transition-colors duration-100`;
  const btnDefault =
    "border-gray-400 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 cursor-pointer";
  const btnActiveAnswer =
    "border-blue-500 dark:border-blue-400 bg-blue-100 dark:bg-blue-900/50 hover:bg-blue-200 dark:hover:bg-blue-900/70 cursor-pointer";
  const btnActiveCandidate =
    "border-teal-500 dark:border-teal-400 bg-teal-100 dark:bg-teal-900/50 hover:bg-teal-200 dark:hover:bg-teal-900/70 cursor-pointer";
  const btnDisabled =
    "border-gray-300 dark:border-slate-700 bg-gray-100 dark:bg-slate-900 text-gray-300 dark:text-slate-600 cursor-not-allowed";

  // Answer row
  const answerButtons: React.ReactNode[] = [];
  for (let i = 1; i <= n; i++) {
    const remaining = n - (placedCounts.get(i) ?? 0);
    const isActive = currentValue === i;
    answerButtons.push(
      <div key={i} className="flex flex-col items-center">
        <RemainingBars remaining={remaining} />
        <button
          className={`${btnBase} font-bold ${isActive ? btnActiveAnswer : btnDefault}`}
          onClick={() => onAnswer(i)}
        >
          {i}
        </button>
      </div>,
    );
  }
  answerButtons.push(
    <div key="clear" className="flex flex-col justify-end">
      <button
        className={`${btnBase} text-xl text-red-600 dark:text-red-400 ${btnDefault}`}
        onClick={onClearAnswer}
      >
        ×
      </button>
    </div>,
  );

  // Memo row
  const memoButtons: React.ReactNode[] = [];
  for (let i = 1; i <= n; i++) {
    const isActive = !memoDisabled && (currentCandidates?.has(i) ?? false);
    memoButtons.push(
      <button
        key={i}
        className={`${btnBase} font-light ${memoDisabled ? btnDisabled : isActive ? btnActiveCandidate : `${btnDefault} text-gray-500 dark:text-slate-400`}`}
        disabled={memoDisabled}
        onClick={() => onToggleCandidate(i)}
      >
        {i}
      </button>,
    );
  }
  memoButtons.push(
    <button
      key="clear"
      className={`${btnBase} flex items-center justify-center ${memoDisabled ? `${btnDisabled}` : `${btnDefault} text-red-600 dark:text-red-400`}`}
      disabled={memoDisabled}
      onClick={onClearCandidates}
    >
      <EraserIcon />
    </button>,
  );

  return (
    <div className="flex flex-col items-center gap-1.5 my-5">
      <div className="flex gap-2 items-end justify-center flex-wrap max-w-[90vw]">
        {answerButtons}
      </div>
      <div className="flex gap-2 justify-center flex-wrap max-w-[90vw]">
        {memoButtons}
      </div>
    </div>
  );
}
