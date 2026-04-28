interface BoardCellProps {
  value: number | null;
  given: boolean;
  candidates: Set<number>;
  selected: boolean;
  sameValue: boolean;
  sameRowOrCol: boolean;
  hasError: boolean;
  completed: boolean;
  hintTarget?: boolean;
  hintLine?: boolean;
  row: number;
  col: number;
  n: number;
  onClick: () => void;
}

function CandidatesGrid({
  candidates,
  n,
}: {
  candidates: Set<number>;
  n: number;
}) {
  if (candidates.size === 0) return null;

  const cols = n <= 6 ? n : Math.ceil(n / 2);

  return (
    <div
      className="grid items-center justify-items-center leading-none"
      style={{
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gridTemplateRows: n > 6 ? "repeat(2, 1fr)" : "1fr",
      }}
    >
      {Array.from({ length: (n > 6 ? 2 : 1) * cols }, (_, i) => {
        const num = i + 1;
        return (
          <span
            key={i}
            className="text-candidate text-gray-500 dark:text-slate-400"
          >
            {num <= n && candidates.has(num) ? num : ""}
          </span>
        );
      })}
    </div>
  );
}

export function BoardCell({
  value,
  given,
  candidates,
  selected,
  sameValue,
  sameRowOrCol,
  hasError,
  completed,
  hintTarget = false,
  hintLine = false,
  row,
  col,
  n,
  onClick,
}: BoardCellProps) {
  const base =
    "cell-size flex items-center justify-center border border-board-border dark:border-board-border-dark transition-colors duration-100 text-xl";

  const bg = completed
    ? "cell-rainbow"
    : hasError && selected
      ? "bg-red-200 dark:bg-red-900/50 ring-2 ring-selected-ring dark:ring-selected-ring-dark ring-inset z-10"
      : hasError
        ? "bg-error-bg dark:bg-error-bg-dark"
        : selected
          ? "bg-selected-bg dark:bg-selected-bg-dark ring-2 ring-selected-ring dark:ring-selected-ring-dark ring-inset z-10"
          : hintTarget
            ? "bg-amber-200 dark:bg-amber-700/50 ring-2 ring-amber-500 ring-inset z-10"
            : hintLine
              ? "bg-amber-100/60 dark:bg-amber-900/30"
              : sameValue
                ? "bg-blue-200 dark:bg-blue-900/40"
                : sameRowOrCol
                  ? "bg-blue-100/60 dark:bg-slate-600/40"
                  : given
                    ? "bg-given-bg dark:bg-given-bg-dark"
                    : "bg-board-bg dark:bg-board-bg-dark";

  const font = given
    ? "font-bold text-gray-800 dark:text-slate-100"
    : "font-normal text-blue-600 dark:text-blue-400";

  const style = completed
    ? ({ "--rainbow-delay": `${(row + col) * 0.15}s` } as React.CSSProperties)
    : undefined;

  return (
    <div
      className={`${base} ${bg} ${font} cursor-pointer`}
      style={style}
      onClick={onClick}
    >
      {value != null ? value : <CandidatesGrid candidates={candidates} n={n} />}
    </div>
  );
}
