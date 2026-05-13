import type { BoardCell as BoardCellType, Puzzle } from "../state/types";
import type { HintResult } from "../engine/types";
import { relevantCells, relevantLines } from "../utils/hint";
import { computeRowColValues } from "../utils/board";
import { ClueCell } from "./ClueCell";
import { BoardCell } from "./BoardCell";

interface PuzzleGridProps {
  puzzle: Puzzle;
  board: BoardCellType[][];
  selectedCell: [number, number] | null;
  highlightValue: number | null;
  errors: Set<string>;
  completed: boolean;
  hint: HintResult | null;
  onCellClick: (row: number, col: number) => void;
}

export function PuzzleGrid({
  puzzle,
  board,
  selectedCell,
  highlightValue,
  errors,
  completed,
  hint,
  onCellClick,
}: PuzzleGridProps) {
  const hintCells = new Set<string>();
  const hintRows = new Set<number>();
  const hintCols = new Set<number>();
  if (hint) {
    relevantCells(hint).forEach(([r, c]) => hintCells.add(`${r},${c}`));
    relevantLines(hint).forEach((line) => {
      if ("row" in line) hintRows.add(line.row);
      else hintCols.add(line.col);
    });
  }
  const { n, clues } = puzzle;
  const { rowVals, colVals } = computeRowColValues(board);
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
      const isSameValue =
        !isSelected &&
        highlightValue !== null &&
        cell.value !== null &&
        cell.value === highlightValue;
      const isSameCandidate =
        !isSelected &&
        highlightValue !== null &&
        cell.value === null &&
        cell.candidates.has(highlightValue) &&
        !rowVals[r].has(highlightValue) &&
        !colVals[c].has(highlightValue);
      const isSameRowOrCol =
        !isSelected &&
        selectedCell !== null &&
        (selectedCell[0] === r || selectedCell[1] === c);

      const inHintCell = hintCells.has(`${r},${c}`);
      const inHintLine = hintRows.has(r) || hintCols.has(c);

      let blocked: Set<number> | undefined;
      if (cell.value === null && !cell.given && cell.candidates.size > 0) {
        blocked = new Set<number>();
        for (const v of rowVals[r]) blocked.add(v);
        for (const v of colVals[c]) blocked.add(v);
      }

      cells.push(
        <BoardCell
          key={key}
          value={cell.value}
          given={cell.given}
          candidates={cell.candidates}
          blocked={blocked}
          selected={isSelected}
          sameValue={isSameValue}
          sameCandidate={isSameCandidate}
          sameRowOrCol={isSameRowOrCol}
          hasError={errors.has(`${r},${c}`)}
          completed={completed}
          hintTarget={inHintCell}
          hintLine={inHintLine && !inHintCell}
          row={r}
          col={c}
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
