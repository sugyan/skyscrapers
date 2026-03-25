interface BoardCellProps {
  value: number | null;
  given: boolean;
  selected: boolean;
  hasError: boolean;
  completed: boolean;
  onClick: () => void;
}

export function BoardCell({
  value,
  given,
  selected,
  hasError,
  completed,
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
      {value ?? ""}
    </div>
  );
}
