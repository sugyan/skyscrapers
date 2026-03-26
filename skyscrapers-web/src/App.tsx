import { useState } from "react";
import { samplePuzzles } from "./samples";
import type { SamplePuzzle } from "./samples";
import { PuzzlePage } from "./components/PuzzlePage";
import type { Puzzle } from "./types";
import "./styles/app.css";

function puzzleFromSample(sample: SamplePuzzle): Puzzle {
  return {
    n: sample.n,
    clues: sample.clues,
    board: sample.board.map((row) =>
      row.map((value) => ({
        value,
        given: value !== null,
      })),
    ),
  };
}

function App() {
  const [selected, setSelected] = useState<SamplePuzzle | null>(null);

  if (selected) {
    return (
      <PuzzlePage
        key={selected.label}
        puzzle={puzzleFromSample(selected)}
        solution={selected.solution}
        onNewPuzzle={() => setSelected(null)}
      />
    );
  }

  return (
    <div className="flex flex-col items-center pt-10 px-5">
      <h1 className="text-2xl font-bold mb-4">Skyscrapers</h1>
      <p className="mb-4">Select a puzzle:</p>
      <ul className="list-none p-0 m-4 space-y-2">
        {samplePuzzles.map((sample) => (
          <li key={sample.label}>
            <button
              className="px-8 py-3 text-lg border border-gray-400 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 cursor-pointer min-w-[200px] hover:bg-gray-200 dark:hover:bg-slate-700"
              onClick={() => setSelected(sample)}
            >
              {sample.label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
