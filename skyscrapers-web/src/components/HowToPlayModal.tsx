import { useEffect } from "react";

interface HowToPlayModalProps {
  onClose: () => void;
}

// 4x4 example puzzle
const EXAMPLE_GRID = [
  [4, 3, 2, 1],
  [2, 1, 4, 3],
  [1, 2, 3, 4],
  [3, 4, 1, 2],
];
const CLUES = {
  top: [2, 3, 1, 2],
  bottom: [3, 1, 4, 2],
  left: [1, 2, 3, 2],
  right: [4, 2, 1, 2],
};
// Highlight row index 1: [2, 1, 4, 3]
const HIGHLIGHT_ROW = 1;

export function HowToPlayModal({ onClose }: HowToPlayModalProps) {
  useEffect(() => {
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        onClose();
      }
    };
    window.addEventListener("keydown", handleKey, true);
    return () => window.removeEventListener("keydown", handleKey, true);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label="How to Play"
    >
      <div
        className="bg-white dark:bg-slate-800 rounded-lg max-w-md w-full max-h-[85vh] overflow-y-auto p-6 relative"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          className="absolute top-3 right-3 text-gray-500 hover:text-gray-800 dark:hover:text-gray-200 text-xl leading-none cursor-pointer"
          aria-label="Close"
        >
          &times;
        </button>

        <h2 className="text-xl font-bold mb-4">How to Play</h2>

        <section className="mb-4">
          <h3 className="font-semibold mb-1">Goal</h3>
          <p className="text-sm leading-relaxed">
            Fill the grid so that each row and column contains every number from
            1 to <em>n</em> exactly once (where <em>n</em> is the grid size).
          </p>
        </section>

        <section className="mb-4">
          <h3 className="font-semibold mb-1">Clues</h3>
          <p className="text-sm leading-relaxed">
            Each number in the grid represents a building of that height. The
            clue numbers around the edges tell you how many buildings are{" "}
            <strong>visible</strong> when looking along that row or column from
            that direction. A taller building hides all shorter buildings behind
            it.
          </p>
        </section>

        <section className="mb-4">
          <h3 className="font-semibold mb-2">Example</h3>
          <div className="flex justify-center mb-3">
            <ExampleGrid />
          </div>
          <p className="text-sm leading-relaxed">
            Look at the highlighted row{" "}
            <strong className="font-mono">[2, 1, 4, 3]</strong> from the left
            (clue = <strong>2</strong>):
          </p>
          <ul className="text-sm leading-relaxed list-disc ml-5 mt-1">
            <li>
              <strong>2</strong> — visible (first building)
            </li>
            <li>
              <strong>1</strong> — hidden behind 2
            </li>
            <li>
              <strong>4</strong> — visible (taller than 2)
            </li>
            <li>
              <strong>3</strong> — hidden behind 4
            </li>
          </ul>
          <p className="text-sm mt-1">
            2 buildings are visible, matching the clue.
          </p>
        </section>

        <button
          onClick={onClose}
          className="w-full mt-2 px-4 py-2 text-sm font-medium border border-gray-400 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700"
        >
          Got it!
        </button>
      </div>
    </div>
  );
}

function ExampleGrid() {
  const n = 4;
  const cellSize = "w-9 h-9";
  const clueSize = "w-9 h-9";

  const clueCell = (value: number | null) => (
    <div
      className={`${clueSize} flex items-center justify-center text-xs text-gray-500 dark:text-gray-400 font-medium`}
    >
      {value ?? ""}
    </div>
  );

  const boardCell = (value: number, highlight: boolean) => (
    <div
      className={`${cellSize} flex items-center justify-center border border-gray-400 dark:border-slate-500 text-sm font-bold ${
        highlight
          ? "bg-blue-100 dark:bg-blue-900/40"
          : "bg-white dark:bg-slate-700"
      }`}
    >
      {value}
    </div>
  );

  return (
    <div
      className="inline-grid gap-0"
      style={{
        gridTemplateColumns: `repeat(${n + 2}, auto)`,
      }}
    >
      {/* Top clue row */}
      <div />
      {CLUES.top.map((v, i) => (
        <div key={`t${i}`}>{clueCell(v)}</div>
      ))}
      <div />

      {/* Board rows with left/right clues */}
      {EXAMPLE_GRID.map((row, r) => (
        <div key={`r${r}`} className="contents">
          {clueCell(CLUES.left[r])}
          {row.map((v, c) => (
            <div key={`c${r}${c}`}>
              {boardCell(v, r === HIGHLIGHT_ROW)}
            </div>
          ))}
          {clueCell(CLUES.right[r])}
        </div>
      ))}

      {/* Bottom clue row */}
      <div />
      {CLUES.bottom.map((v, i) => (
        <div key={`b${i}`}>{clueCell(v)}</div>
      ))}
      <div />
    </div>
  );
}
