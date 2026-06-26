import { HighlightSelector } from "./HighlightSelector";

interface GameControlsProps {
  n: number;
  highlightValue: number | null;
  onHighlightChange: (next: number | null) => void;
  canUndo: boolean;
  onUndo: () => void;
  onReset: () => void;
  onHint: () => void;
  onCheck: () => void;
  onFillCandidates: () => void;
}

export function GameControls({
  n,
  highlightValue,
  onHighlightChange,
  canUndo,
  onUndo,
  onReset,
  onHint,
  onCheck,
  onFillCandidates,
}: GameControlsProps) {
  const primaryBtn =
    "px-4 py-2 text-base border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed touch-manipulation select-none";
  const secondaryBtn =
    "px-3 py-1.5 text-sm border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed touch-manipulation select-none";

  return (
    <div className="flex flex-col gap-2 my-4 w-full max-w-md">
      <div className="flex items-stretch gap-2">
        <button
          className={`${primaryBtn} flex-1`}
          onClick={onUndo}
          disabled={!canUndo}
          aria-label="Undo"
        >
          Undo
        </button>
        <button className={`${primaryBtn} flex-1`} onClick={onCheck}>
          Check
        </button>
        <HighlightSelector
          n={n}
          value={highlightValue}
          onChange={onHighlightChange}
        />
      </div>
      <div className="grid grid-cols-3 gap-2">
        <button className={secondaryBtn} onClick={onHint}>
          Hint
        </button>
        <button className={secondaryBtn} onClick={onFillCandidates}>
          Fill memo
        </button>
        <button className={secondaryBtn} onClick={onReset}>
          Reset
        </button>
      </div>
    </div>
  );
}
