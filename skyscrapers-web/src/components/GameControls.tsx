interface GameControlsProps {
  canUndo: boolean;
  onUndo: () => void;
  onReset: () => void;
  onHint: () => void;
  onCheck: () => void;
  onFillCandidates: () => void;
}

export function GameControls({
  canUndo,
  onUndo,
  onReset,
  onHint,
  onCheck,
  onFillCandidates,
}: GameControlsProps) {
  const primaryBtn =
    "px-4 py-2 text-base border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700";
  const secondaryBtn =
    "px-3 py-1.5 text-sm border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed";

  return (
    <div className="flex flex-col gap-2 my-4 w-full max-w-md">
      <div className="grid grid-cols-2 gap-2">
        <button className={primaryBtn} onClick={onHint}>
          Hint
        </button>
        <button className={primaryBtn} onClick={onCheck}>
          Check
        </button>
      </div>
      <div className="grid grid-cols-3 gap-2">
        <button
          className={secondaryBtn}
          onClick={onUndo}
          disabled={!canUndo}
          aria-label="Undo"
        >
          Undo
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
