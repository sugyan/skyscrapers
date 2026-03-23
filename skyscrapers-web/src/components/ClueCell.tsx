import type { ClueValue } from "../types";

interface ClueCellProps {
  value: ClueValue;
}

export function ClueCell({ value }: ClueCellProps) {
  return (
    <div className="cell-size flex items-center justify-center text-xl text-gray-500 dark:text-slate-400 font-medium">
      {value ?? ""}
    </div>
  );
}
