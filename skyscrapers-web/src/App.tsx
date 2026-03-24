import { useState, useMemo } from "react";
import { decodePuzzle } from "./encoding";
import { samplePuzzles } from "./samples";
import { PuzzlePage } from "./components/PuzzlePage";
import type { Puzzle } from "./types";
import "./styles/app.css";

function App() {
  const puzzleFromUrl = useMemo(() => {
    const params = new URLSearchParams(window.location.search);
    const p = params.get("p");
    if (p) {
      try {
        return decodePuzzle(p);
      } catch {
        return null;
      }
    }
    return null;
  }, []);

  const [puzzle, setPuzzle] = useState<Puzzle | null>(puzzleFromUrl);

  if (puzzle) {
    return (
      <PuzzlePage
        key={JSON.stringify(puzzle)}
        puzzle={puzzle}
        onNewPuzzle={() => setPuzzle(null)}
      />
    );
  }

  return (
    <div className="flex flex-col items-center pt-10 px-5">
      <h1 className="text-2xl font-bold mb-4">Skyscrapers</h1>
      <p className="mb-4">Select a puzzle:</p>
      <ul className="list-none p-0 m-4 space-y-2">
        {samplePuzzles.map((sample) => (
          <li key={sample.encoded}>
            <button
              className="px-8 py-3 text-lg border border-gray-400 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 cursor-pointer min-w-[200px] hover:bg-gray-200 dark:hover:bg-slate-700"
              onClick={() => setPuzzle(decodePuzzle(sample.encoded))}
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
