import { useState, useMemo } from "react";
import { decodePuzzle } from "./encoding";
import { samplePuzzles } from "./samples";
import { PuzzlePage } from "./components/PuzzlePage";
import type { Puzzle } from "./types";
import "./styles/puzzle.css";
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
    <div className="sample-menu">
      <h1>Skyscrapers</h1>
      <p>Select a puzzle:</p>
      <ul>
        {samplePuzzles.map((sample) => (
          <li key={sample.encoded}>
            <button onClick={() => setPuzzle(decodePuzzle(sample.encoded))}>
              {sample.label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
