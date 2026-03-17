interface BoardCellProps {
  value: number | null;
  given: boolean;
  selected: boolean;
  hasError: boolean;
  onClick: () => void;
}

export function BoardCell({
  value,
  given,
  selected,
  hasError,
  onClick,
}: BoardCellProps) {
  const classNames = ["board-cell"];
  if (given) classNames.push("given");
  if (selected) classNames.push("selected");
  if (hasError) classNames.push("error");

  return (
    <div
      className={classNames.join(" ")}
      onClick={given ? undefined : onClick}
    >
      {value ?? ""}
    </div>
  );
}
