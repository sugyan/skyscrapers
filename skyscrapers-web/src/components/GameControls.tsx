interface GameControlsProps {
  onReset: () => void;
  onHint: () => void;
  onCheck: () => void;
  onFillCandidates: () => void;
  onNewPuzzle: () => void;
}

export function GameControls({
  onReset,
  onHint,
  onCheck,
  onFillCandidates,
  onNewPuzzle,
}: GameControlsProps) {
  const btnClass =
    "px-5 py-2 text-base border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700";

  return (
    <div className="flex gap-3 flex-wrap justify-center items-center my-4">
      <button className={btnClass} onClick={onReset}>
        Reset
      </button>
      <button className={btnClass} onClick={onFillCandidates}>
        Fill candidates
      </button>
      <button className={btnClass} onClick={onHint}>
        Hint
      </button>
      <button className={btnClass} onClick={onCheck}>
        Check
      </button>
      <button className={btnClass} onClick={onNewPuzzle}>
        New Puzzle
      </button>
    </div>
  );
}
