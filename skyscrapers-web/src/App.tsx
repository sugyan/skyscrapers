import { useEffect, useMemo, useState } from "react";
import {
  DIFFICULTIES,
  Player,
  normalizeDifficultyParam,
  type Difficulty,
  type Puzzle,
} from "skyscrapers-player";
import { WasmEngine } from "./engine/wasm-engine";
import "skyscrapers-player/styles.css";
import { HowToPlayModal } from "./components/HowToPlayModal";

function parseUrlParams(): {
  n: number;
  seed: bigint;
  difficulty?: Difficulty;
} | null {
  const params = new URLSearchParams(window.location.search);
  const nStr = params.get("n");
  const seedStr = params.get("seed");
  if (!nStr || !seedStr) return null;
  const n = Number(nStr);
  if (!Number.isInteger(n) || n < 4 || n > 8) return null;
  try {
    const seed = BigInt(seedStr);
    const difficulty = normalizeDifficultyParam(params.get("difficulty"));
    return { n, seed, difficulty };
  } catch {
    return null;
  }
}

function updateUrl(n: number, seed: string, difficulty: Difficulty | null) {
  const url = new URL(window.location.href);
  url.searchParams.set("n", String(n));
  url.searchParams.set("seed", seed);
  if (difficulty) {
    url.searchParams.set("difficulty", difficulty);
  } else {
    url.searchParams.delete("difficulty");
  }
  window.history.pushState({}, "", url);
}

function clearUrl() {
  const url = new URL(window.location.href);
  url.search = "";
  window.history.pushState({}, "", url);
}

const SIZES = [4, 5, 6, 7, 8] as const;

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function formatGenerateError(
  e: unknown,
  n: number,
  difficulty: Difficulty | null,
): string {
  const message = e instanceof Error ? e.message : String(e);
  if (difficulty && message.includes("failed to generate puzzle at target")) {
    const attemptsMatch = message.match(/(\d+)\s+attempts?/i);
    const attemptsText = attemptsMatch
      ? `${attemptsMatch[1]} attempts`
      : "the configured number of attempts";
    return `Couldn't find a ${difficulty} puzzle for size ${n} in ${attemptsText}. Try another seed, a different size, or a lower difficulty.`;
  }
  return message;
}

function App() {
  const engine = useMemo(() => new WasmEngine(), []);
  // URL params are parsed once on first render and threaded into the
  // initial state of the form fields. Deriving the initial values here
  // instead of inside an effect avoids the `react-hooks/set-state-in-effect`
  // warning (cascading renders from synchronous setState in an effect)
  // and keeps the effect below focused on its actual side effect: the
  // async puzzle-generation request.
  const initialParams = useMemo(() => parseUrlParams(), []);
  const [current, setCurrent] = useState<{
    puzzle: Puzzle;
    solution: number[][];
  } | null>(null);
  const [generating, setGenerating] = useState<boolean>(initialParams !== null);
  const [size, setSize] = useState<number>(initialParams?.n ?? 5);
  const [seedInput, setSeedInput] = useState<string>(
    initialParams ? initialParams.seed.toString() : "",
  );
  const [difficulty, setDifficulty] = useState<Difficulty | "">(
    initialParams?.difficulty ?? "",
  );
  const [lastSeed, setLastSeed] = useState<string | null>(null);
  const [lastDifficulty, setLastDifficulty] = useState<Difficulty | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showHowToPlay, setShowHowToPlay] = useState(false);

  // Generate from URL params on initial load. The effect body itself
  // performs no synchronous setState — all updates happen in async
  // callbacks once the engine call settles.
  useEffect(() => {
    if (!initialParams) return;
    engine
      .generatePuzzle(
        initialParams.n,
        initialParams.seed,
        initialParams.difficulty,
      )
      .then((result) => {
        setLastSeed(initialParams.seed.toString());
        setLastDifficulty(initialParams.difficulty ?? null);
        setCurrent(result);
      })
      .catch((e) => {
        setError(
          formatGenerateError(
            e,
            initialParams.n,
            initialParams.difficulty ? initialParams.difficulty : null,
          ),
        );
      })
      .finally(() => setGenerating(false));
  }, [engine, initialParams]);

  const handleGenerate = async () => {
    setGenerating(true);
    setError(null);
    const target: Difficulty | null = difficulty || null;
    try {
      const seed = seedInput.trim()
        ? BigInt(seedInput.trim())
        : engine.randomSeed();
      const seedStr = seed.toString();
      setLastSeed(seedStr);
      const result = await engine.generatePuzzle(
        size,
        seed,
        target ?? undefined,
      );
      setLastDifficulty(target);
      setCurrent(result);
      updateUrl(size, seedStr, target);
    } catch (e) {
      setError(formatGenerateError(e, size, target));
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
      <>
        <Player
          key={`${current.puzzle.n}-${lastSeed}`}
          puzzle={current.puzzle}
          solution={current.solution}
          engine={engine}
          difficulty={lastDifficulty}
          onNewPuzzle={handleNewPuzzle}
          onShowHowToPlay={() => setShowHowToPlay(true)}
        />
        {showHowToPlay && (
          <HowToPlayModal onClose={() => setShowHowToPlay(false)} />
        )}
      </>
    );
  }

  return (
    <div className="flex flex-col items-center pt-10 px-5">
      <h1 className="text-2xl font-bold mb-3">Skyscrapers</h1>
      <button
        onClick={() => setShowHowToPlay(true)}
        className="mb-6 text-sm text-blue-600 dark:text-blue-400 underline cursor-pointer hover:text-blue-800 dark:hover:text-blue-300"
      >
        How to Play
      </button>
      {showHowToPlay && (
        <HowToPlayModal onClose={() => setShowHowToPlay(false)} />
      )}

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
        <div className="flex items-center gap-3 mb-3">
          <label htmlFor="difficulty-select" className="text-sm">
            Difficulty:
          </label>
          <select
            id="difficulty-select"
            value={difficulty}
            onChange={(e) => setDifficulty(e.target.value as Difficulty | "")}
            className="px-3 py-1.5 border border-gray-400 dark:border-slate-600 rounded bg-white dark:bg-slate-800"
          >
            <option value="">Any</option>
            {DIFFICULTIES.map((d) => (
              <option key={d} value={d}>
                {capitalize(d)}
              </option>
            ))}
          </select>
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
          <p className="mt-2 text-xs text-gray-500">
            Last seed: {lastSeed}
            {lastDifficulty ? ` · ${capitalize(lastDifficulty)}` : ""}
          </p>
        )}
      </section>
    </div>
  );
}

export default App;
