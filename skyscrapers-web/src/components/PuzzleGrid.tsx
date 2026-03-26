import type { BoardCell as BoardCellType, Puzzle } from "../types";
import { ClueCell } from "./ClueCell";
import { BoardCell } from "./BoardCell";

interface PuzzleGridProps {
  puzzle: Puzzle;
  board: BoardCellType[][];
  selectedCell: [number, number] | null;
  errors: Set<string>;
  completed: boolean;
  onCellClick: (row: number, col: number) => void;
}

export function PuzzleGrid({
  puzzle,
  board,
  selectedCell,
  errors,
  completed,
  onCellClick,
}: PuzzleGridProps) {
  const { n, clues } = puzzle;
  const cells: React.ReactNode[] = [];

  for (let gridRow = 0; gridRow < n + 2; gridRow++) {
    for (let gridCol = 0; gridCol < n + 2; gridCol++) {
      const key = `${gridRow}-${gridCol}`;

      // Corner cells
      if (
        (gridRow === 0 || gridRow === n + 1) &&
        (gridCol === 0 || gridCol === n + 1)
      ) {
        cells.push(<div key={key} className="cell-size" />);
        continue;
      }

      // Top clues
      if (gridRow === 0 && gridCol >= 1 && gridCol <= n) {
        cells.push(
          <ClueCell key={key} value={clues.top[gridCol - 1]} direction="top" />,
        );
        continue;
      }

      // Bottom clues
      if (gridRow === n + 1 && gridCol >= 1 && gridCol <= n) {
        cells.push(
          <ClueCell
            key={key}
            value={clues.bottom[gridCol - 1]}
            direction="bottom"
          />,
        );
        continue;
      }

      // Left clues
      if (gridCol === 0 && gridRow >= 1 && gridRow <= n) {
        cells.push(
          <ClueCell
            key={key}
            value={clues.left[gridRow - 1]}
            direction="left"
          />,
        );
        continue;
      }

      // Right clues
      if (gridCol === n + 1 && gridRow >= 1 && gridRow <= n) {
        cells.push(
          <ClueCell
            key={key}
            value={clues.right[gridRow - 1]}
            direction="right"
          />,
        );
        continue;
      }

      // Board cells
      const r = gridRow - 1;
      const c = gridCol - 1;
      const cell = board[r][c];
      const isSelected =
        selectedCell !== null && selectedCell[0] === r && selectedCell[1] === c;

      cells.push(
        <BoardCell
          key={key}
          value={cell.value}
          given={cell.given}
          candidates={cell.candidates}
          selected={isSelected}
          hasError={errors.has(`${r},${c}`)}
          completed={completed}
          n={n}
          onClick={() => onCellClick(r, c)}
        />,
      );
    }
  }

  return (
    <div
      className="grid gap-0 w-fit mx-auto select-none"
      style={
        {
          gridTemplateColumns: `auto repeat(${n}, 1fr) auto`,
          "--grid-cols": n + 2,
        } as React.CSSProperties
      }
    >
      {cells}
    </div>
  );
}
