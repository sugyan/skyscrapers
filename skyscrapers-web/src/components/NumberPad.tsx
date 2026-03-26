import type { InputMode } from "../types";

interface NumberPadProps {
  n: number;
  inputMode: InputMode;
  currentValue: number | null;
  currentCandidates: Set<number> | null;
  onNumberSelect: (value: number) => void;
  onClear: () => void;
  onModeChange: (mode: InputMode) => void;
}

export function NumberPad({
  n,
  inputMode,
  currentValue,
  currentCandidates,
  onNumberSelect,
  onClear,
  onModeChange,
}: NumberPadProps) {
  const btnBase =
    "w-12 h-12 text-lg border rounded-md cursor-pointer transition-colors duration-100";
  const btnDefault =
    "border-gray-400 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700";
  const btnActiveAnswer =
    "border-blue-500 dark:border-blue-400 bg-blue-100 dark:bg-blue-900/50 hover:bg-blue-200 dark:hover:bg-blue-900/70";
  const btnActiveCandidate =
    "border-teal-500 dark:border-teal-400 bg-teal-100 dark:bg-teal-900/50 hover:bg-teal-200 dark:hover:bg-teal-900/70";

  const isCandidate = inputMode === "candidate";

  const buttons: React.ReactNode[] = [];
  for (let i = 1; i <= n; i++) {
    let isActive: boolean;
    if (isCandidate) {
      isActive = currentCandidates?.has(i) ?? false;
    } else {
      isActive = currentValue === i;
    }
    const activeStyle = isCandidate ? btnActiveCandidate : btnActiveAnswer;

    buttons.push(
      <button
        key={i}
        className={`${btnBase} ${isActive ? activeStyle : btnDefault}`}
        onClick={() => onNumberSelect(i)}
      >
        {i}
      </button>,
    );
  }

  buttons.push(
    <button
      key="clear"
      className={`${btnBase} ${btnDefault} text-xl text-red-600 dark:text-red-400`}
      onClick={onClear}
    >
      ×
    </button>,
  );

  const modeBtnBase =
    "px-3 h-10 text-sm border rounded-md cursor-pointer transition-colors duration-100";
  const modeBtnAnswer =
    "border-blue-500 dark:border-blue-400 bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300";
  const modeBtnCandidate =
    "border-teal-500 dark:border-teal-400 bg-teal-100 dark:bg-teal-900/50 text-teal-700 dark:text-teal-300";

  return (
    <div className="flex flex-col items-center gap-2 my-5">
      <div className="flex gap-2">
        <button
          className={`${modeBtnBase} ${!isCandidate ? modeBtnAnswer : btnDefault}`}
          onClick={() => onModeChange("answer")}
        >
          Answer
        </button>
        <button
          className={`${modeBtnBase} ${isCandidate ? modeBtnCandidate : btnDefault}`}
          onClick={() => onModeChange("candidate")}
        >
          Memo
        </button>
      </div>
      <div className="flex gap-2 justify-center flex-wrap max-w-[90vw]">
        {buttons}
      </div>
    </div>
  );
}
