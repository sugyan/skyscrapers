interface NumberPadProps {
  n: number;
  currentValue: number | null;
  onNumberSelect: (value: number) => void;
  onClear: () => void;
}

export function NumberPad({
  n,
  currentValue,
  onNumberSelect,
  onClear,
}: NumberPadProps) {
  const btnBase =
    "w-12 h-12 text-lg border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer transition-colors duration-100 hover:bg-gray-200 dark:hover:bg-slate-700";
  const btnActive = "bg-blue-100 dark:bg-blue-900/50 border-blue-500 dark:border-blue-400";

  const buttons: React.ReactNode[] = [];

  for (let i = 1; i <= n; i++) {
    buttons.push(
      <button
        key={i}
        className={`${btnBase} ${currentValue === i ? btnActive : ""}`}
        onClick={() => onNumberSelect(i)}
      >
        {i}
      </button>
    );
  }

  buttons.push(
    <button
      key="clear"
      className={`${btnBase} text-xl text-red-600 dark:text-red-400`}
      onClick={onClear}
    >
      ×
    </button>
  );

  return (
    <div className="flex gap-2 justify-center my-5 flex-wrap max-w-[90vw]">
      {buttons}
    </div>
  );
}
