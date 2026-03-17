import type { ClueValue } from "../types";

interface ClueCellProps {
  value: ClueValue;
}

export function ClueCell({ value }: ClueCellProps) {
  return <div className="clue-cell">{value ?? ""}</div>;
}
