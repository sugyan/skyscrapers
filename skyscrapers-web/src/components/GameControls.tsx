interface GameControlsProps {
  onReset: () => void;
  onCheck: () => void;
  onNewPuzzle: () => void;
  completed: boolean;
}

export function GameControls({
  onReset,
  onCheck,
  onNewPuzzle,
  completed,
}: GameControlsProps) {
  const btnClass =
    "px-5 py-2 text-base border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700";

  return (
    <div className="flex gap-3 justify-center items-center my-4">
      <button className={btnClass} onClick={onReset}>
        Reset
      </button>
      <button className={btnClass} onClick={onCheck}>
        Check
      </button>
      <button className={btnClass} onClick={onNewPuzzle}>
        New Puzzle
      </button>
      {completed && (
        <span className="text-green-600 dark:text-green-400 font-bold text-lg">
          Completed!
        </span>
      )}
    </div>
  );
}
