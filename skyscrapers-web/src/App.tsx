import { useEffect, useState } from "react";
import { samplePuzzles } from "./samples";
import type { SamplePuzzle } from "./samples";
import { PuzzlePage } from "./components/PuzzlePage";
import type { Puzzle } from "./types";
import { generatePuzzle, randomSeed } from "./wasm";
import type { GenerateResult } from "./wasm";
import "./styles/app.css";

function puzzleFromSample(sample: SamplePuzzle): GenerateResult {
  return {
    puzzle: {
      n: sample.n,
      clues: sample.clues,
      board: sample.board.map((row) =>
        row.map((value) => ({
          value,
          given: value !== null,
          candidates: new Set<number>(),
        })),
      ),
    },
    solution: sample.solution,
  };
}

function parseUrlParams(): { n: number; seed: bigint } | null {
  const params = new URLSearchParams(window.location.search);
  const nStr = params.get("n");
  const seedStr = params.get("seed");
  if (!nStr || !seedStr) return null;
  const n = Number(nStr);
  if (!Number.isInteger(n) || n < 4 || n > 8) return null;
  try {
    return { n, seed: BigInt(seedStr) };
  } catch {
    return null;
  }
}

function updateUrl(n: number, seed: string) {
  const url = new URL(window.location.href);
  url.searchParams.set("n", String(n));
  url.searchParams.set("seed", seed);
  window.history.pushState({}, "", url);
}

function clearUrl() {
  const url = new URL(window.location.href);
  url.search = "";
  window.history.pushState({}, "", url);
}

const SIZES = [4, 5, 6, 7, 8] as const;

function App() {
  const [current, setCurrent] = useState<{
    puzzle: Puzzle;
    solution: number[][];
  } | null>(null);
  const [generating, setGenerating] = useState(false);
  const [size, setSize] = useState<number>(5);
  const [seedInput, setSeedInput] = useState("");
  const [lastSeed, setLastSeed] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Generate from URL params on initial load
  useEffect(() => {
    const params = parseUrlParams();
    if (params) {
      setSize(params.n);
      setSeedInput(params.seed.toString());
      setGenerating(true);
      generatePuzzle(params.n, params.seed)
        .then((result) => {
          setLastSeed(params.seed.toString());
          setCurrent(result);
        })
        .catch((e) => {
          setError(e instanceof Error ? e.message : String(e));
        })
        .finally(() => setGenerating(false));
    }
  }, []);

  const handleGenerate = async () => {
    setGenerating(true);
    setError(null);
    try {
      const seed = seedInput.trim() ? BigInt(seedInput.trim()) : randomSeed();
      const seedStr = seed.toString();
      setLastSeed(seedStr);
      const result = await generatePuzzle(size, seed);
      setCurrent(result);
      updateUrl(size, seedStr);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setGenerating(false);
    }
  };

  const handleNewPuzzle = () => {
    setCurrent(null);
    clearUrl();
  };

  if (current) {
    return (
      <PuzzlePage
        key={`${current.puzzle.n}-${lastSeed}`}
        puzzle={current.puzzle}
        solution={current.solution}
        onNewPuzzle={handleNewPuzzle}
      />
    );
  }

  return (
    <div className="flex flex-col items-center pt-10 px-5">
      <h1 className="text-2xl font-bold mb-6">Skyscrapers</h1>

      <section className="mb-8 w-full max-w-sm">
        <h2 className="text-lg font-semibold mb-3">Generate</h2>
        <div className="flex items-center gap-3 mb-3">
          <label htmlFor="size-select" className="text-sm">
            Size:
          </label>
          <select
            id="size-select"
            value={size}
            onChange={(e) => setSize(Number(e.target.value))}
            className="px-3 py-1.5 border border-gray-400 dark:border-slate-600 rounded bg-white dark:bg-slate-800"
          >
            {SIZES.map((n) => (
              <option key={n} value={n}>
                {n}x{n}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-3 mb-3">
          <label htmlFor="seed-input" className="text-sm">
            Seed:
          </label>
          <input
            id="seed-input"
            type="text"
            value={seedInput}
            onChange={(e) => setSeedInput(e.target.value)}
            placeholder="random"
            className="flex-1 px-3 py-1.5 border border-gray-400 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-sm"
          />
        </div>
        <button
          onClick={handleGenerate}
          disabled={generating}
          className="w-full px-4 py-2 text-sm font-medium border border-gray-400 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {generating ? "Generating..." : "Generate"}
        </button>
        {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
        {lastSeed && !current && (
          <p className="mt-2 text-xs text-gray-500">Last seed: {lastSeed}</p>
        )}
      </section>

      <section className="w-full max-w-sm">
        <h2 className="text-lg font-semibold mb-3">Samples</h2>
        <ul className="list-none p-0 space-y-2">
          {samplePuzzles.map((sample) => (
            <li key={sample.label}>
              <button
                className="w-full px-8 py-3 text-lg border border-gray-400 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700"
                onClick={() => setCurrent(puzzleFromSample(sample))}
              >
                {sample.label}
              </button>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

export default App;
