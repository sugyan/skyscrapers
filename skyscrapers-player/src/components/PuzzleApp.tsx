import { useEffect, useState } from "react";
import { Player } from "./Player";
import { HowToPlayModal } from "./HowToPlayModal";
import { DIFFICULTIES, type Difficulty } from "../engine/types";
import type { SkyscrapersEngine } from "../engine/types";
import type { Puzzle } from "../state/types";

/** A puzzle to generate: grid size, RNG seed, and an optional target difficulty. */
export interface PuzzleRequest {
  n: number;
  seed: bigint;
  difficulty?: Difficulty;
}

export interface PuzzleAppProps {
  /**
   * Engine used for generation and hints. Keep the reference stable across
   * renders (e.g. `useMemo(() => new WasmEngine(), [])`).
   */
  engine: SkyscrapersEngine;
  /**
   * Puzzle to generate automatically on first render — used by hosts that
   * restore a puzzle from somewhere outside the component (the web app reads
   * it from the URL query string). Only the value present on the first render
   * is read; later changes are ignored.
   */
  initialRequest?: PuzzleRequest | null;
  /**
   * Called after the user generates a puzzle from the form, so the host can
   * record it (the web app writes it back to the URL). Not called for
   * `initialRequest`, whose parameters the host already knows.
   */
  onGenerated?: (
    n: number,
    seed: string,
    difficulty: Difficulty | null,
  ) => void;
  /** Called when the user leaves a puzzle via "New Puzzle". */
  onCleared?: () => void;
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

/**
 * The full single-puzzle app: generation form, `<Player>`, and the How to Play
 * modal.
 *
 * Platform differences stay in the host via plain callbacks, so this component
 * needs no knowledge of URLs, `window.history`, or which transport the engine
 * speaks. `skyscrapers-web` supplies URL read/write; the Tauri app supplies
 * nothing at all.
 */
export function PuzzleApp({
  engine,
  initialRequest,
  onGenerated,
  onCleared,
}: PuzzleAppProps) {
  // Snapshot the initial request on the first render, via a lazy state
  // initializer that is never updated. This keeps the generation effect below
  // from re-firing if the host passes a freshly-built object on every render,
  // and lets the form fields derive their initial values here rather than via
  // setState inside an effect (which would trip
  // `react-hooks/set-state-in-effect` and cascade renders).
  const [request] = useState<PuzzleRequest | null>(
    () => initialRequest ?? null,
  );

  const [current, setCurrent] = useState<{
    puzzle: Puzzle;
    solution: number[][];
  } | null>(null);
  const [generating, setGenerating] = useState<boolean>(request !== null);
  const [size, setSize] = useState<number>(request?.n ?? 5);
  const [seedInput, setSeedInput] = useState<string>(
    request ? request.seed.toString() : "",
  );
  const [difficulty, setDifficulty] = useState<Difficulty | "">(
    request?.difficulty ?? "",
  );
  const [lastSeed, setLastSeed] = useState<string | null>(null);
  const [lastDifficulty, setLastDifficulty] = useState<Difficulty | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showHowToPlay, setShowHowToPlay] = useState(false);

  // Generate the initial request on mount. The effect body itself performs no
  // synchronous setState — all updates happen in async callbacks once the
  // engine call settles.
  useEffect(() => {
    if (!request) return;
    engine
      .generatePuzzle(request.n, request.seed, request.difficulty)
      .then((result) => {
        setLastSeed(request.seed.toString());
        // Label reflects the puzzle's actual (solver-detected) difficulty, not
        // the requested target — so "Any" generation still gets a label.
        setLastDifficulty(result.difficulty);
        setCurrent(result);
      })
      .catch((e) => {
        setError(
          formatGenerateError(
            e,
            request.n,
            request.difficulty ? request.difficulty : null,
          ),
        );
      })
      .finally(() => setGenerating(false));
  }, [engine, request]);

  const handleGenerate = async () => {
    setGenerating(true);
    setError(null);
    // Clear the previous label so a failed generation can't show the new seed
    // alongside a stale difficulty from an earlier success.
    setLastDifficulty(null);
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
      // Label reflects the puzzle's actual (solver-detected) difficulty; the
      // host is told the requested target so a restored request reproduces it.
      setLastDifficulty(result.difficulty);
      setCurrent(result);
      onGenerated?.(size, seedStr, target);
    } catch (e) {
      setError(formatGenerateError(e, size, target));
    } finally {
      setGenerating(false);
    }
  };

  const handleNewPuzzle = () => {
    setCurrent(null);
    onCleared?.();
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
        {lastSeed && (
          <p className="text-center text-xs text-gray-500 dark:text-gray-400 -mt-2 mb-6">
            Seed: {lastSeed}
          </p>
        )}
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
          <p className="mt-2 text-xs text-gray-500">Last seed: {lastSeed}</p>
        )}
      </section>
    </div>
  );
}
