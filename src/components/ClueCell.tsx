import type { ClueValue } from "../state/types";

type Direction = "top" | "bottom" | "left" | "right";

const arrows: Record<Direction, string> = {
  top: "↓",
  bottom: "↑",
  left: "→",
  right: "←",
};

interface ClueCellProps {
  value: ClueValue;
  direction: Direction;
}

export function ClueCell({ value, direction }: ClueCellProps) {
  if (value === null) {
    return <div className="cell-size" />;
  }

  const isVertical = direction === "top" || direction === "bottom";
  const arrowEl = (
    <span className="text-xs opacity-50">{arrows[direction]}</span>
  );

  return (
    <div
      className={`cell-size flex items-center justify-center gap-0.5 text-gray-500 dark:text-slate-400 font-medium ${isVertical ? "flex-col" : ""}`}
    >
      {(direction === "bottom" || direction === "right") && arrowEl}
      <span className="text-xl">{value}</span>
      {(direction === "top" || direction === "left") && arrowEl}
    </div>
  );
}
