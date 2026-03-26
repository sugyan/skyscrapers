interface BoardCellProps {
  value: number | null;
  given: boolean;
  candidates: Set<number>;
  selected: boolean;
  hasError: boolean;
  completed: boolean;
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

  const cols = n <= 5 ? n : 3;
  const rows = Math.ceil(n / cols);

  return (
    <div
      className="grid w-full h-full items-center justify-items-center leading-none"
      style={{
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gridTemplateRows: `repeat(${rows}, 1fr)`,
      }}
    >
      {Array.from({ length: rows * cols }, (_, i) => {
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
  hasError,
  completed,
  n,
  onClick,
}: BoardCellProps) {
  const base =
    "cell-size flex items-center justify-center border border-board-border dark:border-board-border-dark transition-colors duration-100 text-xl";

  const bg = completed
    ? given
      ? "bg-completed-given-bg dark:bg-completed-given-bg-dark transition-colors duration-500"
      : "bg-completed-bg dark:bg-completed-bg-dark transition-colors duration-500"
    : hasError && selected
      ? "bg-red-200 dark:bg-red-900/50 ring-2 ring-selected-ring dark:ring-selected-ring-dark ring-inset z-10"
      : hasError
        ? "bg-error-bg dark:bg-error-bg-dark"
        : selected
          ? "bg-selected-bg dark:bg-selected-bg-dark ring-2 ring-selected-ring dark:ring-selected-ring-dark ring-inset z-10"
          : given
            ? "bg-given-bg dark:bg-given-bg-dark"
            : "bg-board-bg dark:bg-board-bg-dark";

  const font = given
    ? "font-bold cursor-default"
    : "font-normal cursor-pointer";

  return (
    <div
      className={`${base} ${bg} ${font}`}
      onClick={given ? undefined : onClick}
    >
      {value != null ? (
        value
      ) : (
        <CandidatesGrid candidates={candidates} n={n} />
      )}
    </div>
  );
}
