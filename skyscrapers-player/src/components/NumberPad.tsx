import type { BoardCell } from "../state/types";

interface NumberPadProps {
  n: number;
  board: BoardCell[][];
  currentValue: number | null;
  currentCandidates: Set<number> | null;
  answerDisabled: boolean;
  memoDisabled: boolean;
  onAnswer: (value: number) => void;
  onClearAnswer: () => void;
  onToggleCandidate: (value: number) => void;
  onClearCandidates: () => void;
}

function RemainingBars({ remaining }: { remaining: number }) {
  if (remaining <= 0) return null;
  return (
    <div className="flex flex-col items-center gap-0.5 mb-1.5">
      {Array.from({ length: remaining }, (_, i) => (
        <div
          key={i}
          className="w-4 h-0.5 bg-blue-400 dark:bg-blue-500 rounded-full"
        />
      ))}
    </div>
  );
}

/**
 * Tap-handler bundle that fires `action` on the actual touch/pen hit for
 * touch/pen input, while leaving mouse and programmatic activations on the
 * standard `click` path.
 *
 * Background: on iOS Safari, two rapid taps on adjacent buttons can route the
 * second `click` to the previously-tapped button (gesture / focus heuristics
 * inside WebKit), so a player tapping `4` then `5` sees `4` toggled twice and
 * `5` ignored. Firing on `pointerdown` for touch/pen sidesteps that quirk —
 * the OS hit-test at finger-down time picks the right button.
 *
 * Mouse input is intentionally excluded from the pointerdown path so the
 * desktop convention of "activate on click, cancel by dragging off" still
 * works; mouse activations flow through `onClick` normally.
 *
 * Activation paths covered, matching native button semantics:
 *   - Touch / pen tap → `onPointerDown` (preventDefault suppresses the
 *     synthesized click).
 *   - Mouse click    → `onClick`.
 *   - Enter          → `onKeyDown`, with `e.repeat` ignored so holding the
 *     key down doesn't re-fire.
 *   - Space          → `onKeyUp` (W3C button activation timing).
 *   - Programmatic `.click()` / assistive tech → `onClick` fallback.
 *
 * The handler that actually ran the action sets `suppressClickUntil` so the
 * native click that follows touch/pen/key activation doesn't double-fire the
 * action. The flag lives at module scope (not per-render closure) because
 * React re-renders between event handlers and a fresh closure would lose it.
 */
let suppressClickUntil = 0;

function tapProps(action: () => void, disabled: boolean) {
  if (disabled) return {};
  const suppressNextClick = () => {
    suppressClickUntil = Date.now() + 500;
  };
  return {
    onPointerDown: (e: React.PointerEvent<HTMLButtonElement>) => {
      if (e.button !== 0) return;
      // Mouse keeps the standard click semantics; only touch/pen need the
      // pointerdown shortcut to dodge iOS Safari's click-routing quirk.
      if (e.pointerType !== "touch" && e.pointerType !== "pen") return;
      e.preventDefault();
      suppressNextClick();
      action();
    },
    onKeyDown: (e: React.KeyboardEvent<HTMLButtonElement>) => {
      if (e.key === "Enter" && !e.repeat) {
        e.preventDefault();
        suppressNextClick();
        action();
      } else if (e.key === " ") {
        // Prevent page scroll while Space is held; activation happens on keyup.
        e.preventDefault();
      }
    },
    onKeyUp: (e: React.KeyboardEvent<HTMLButtonElement>) => {
      if (e.key === " ") {
        e.preventDefault();
        suppressNextClick();
        action();
      }
    },
    onClick: () => {
      if (Date.now() <= suppressClickUntil) {
        suppressClickUntil = 0;
        return;
      }
      action();
    },
  };
}

function EraserIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="w-5 h-5"
      aria-hidden="true"
    >
      <path d="M20 20H7L3 16c-.8-.8-.8-2 0-2.8L14.6 1.6c.8-.8 2-.8 2.8 0L21 5.2c.8.8.8 2 0 2.8L12 17" />
      <path d="M6 11l4 4" />
    </svg>
  );
}

export function NumberPad({
  n,
  board,
  currentValue,
  currentCandidates,
  answerDisabled,
  memoDisabled,
  onAnswer,
  onClearAnswer,
  onToggleCandidate,
  onClearCandidates,
}: NumberPadProps) {
  // Count how many of each digit are placed on the board
  const placedCounts = new Map<number, number>();
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      const v = board[r][c].value;
      if (v !== null) {
        placedCounts.set(v, (placedCounts.get(v) ?? 0) + 1);
      }
    }
  }

  // Shrink buttons for larger n so the row still fits on a single line on
  // narrow mobile viewports (n+1 buttons inside max-w-[90vw] ≈ 324px @360px).
  const isLarge = n >= 7;
  const sizeClass =
    n >= 8
      ? "w-8 h-8 sm:w-12 sm:h-12"
      : n === 7
        ? "w-9 h-9 sm:w-12 sm:h-12"
        : "w-10 h-10 sm:w-12 sm:h-12";
  const digitTextClass = n >= 8 ? "text-base sm:text-lg" : "text-lg";
  const clearTextClass = n >= 8 ? "text-lg sm:text-xl" : "text-xl";
  const rowGapClass = isLarge ? "gap-1 sm:gap-2" : "gap-1.5 sm:gap-2";

  const btnBase = `${sizeClass} border rounded-md transition-colors duration-100 touch-manipulation select-none`;
  const btnDefault =
    "border-gray-400 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 cursor-pointer";
  const btnActiveAnswer =
    "border-blue-500 dark:border-blue-400 bg-blue-100 dark:bg-blue-900/50 hover:bg-blue-200 dark:hover:bg-blue-900/70 cursor-pointer";
  const btnActiveCandidate =
    "border-teal-500 dark:border-teal-400 bg-teal-100 dark:bg-teal-900/50 hover:bg-teal-200 dark:hover:bg-teal-900/70 cursor-pointer";
  const btnDisabled =
    "border-gray-300 dark:border-slate-700 bg-gray-100 dark:bg-slate-900 text-gray-300 dark:text-slate-600 cursor-not-allowed";

  // Answer row — value entry only. The row is disabled (via answerDisabled)
  // when there is no editable selected cell — nothing selected, or the
  // selected cell is a given clue; highlighting lives on its own control.
  const answerButtons: React.ReactNode[] = [];
  for (let i = 1; i <= n; i++) {
    const remaining = n - (placedCounts.get(i) ?? 0);
    const isAnswerActive = !answerDisabled && currentValue === i;
    const stateClass = answerDisabled
      ? btnDisabled
      : isAnswerActive
        ? btnActiveAnswer
        : btnDefault;
    answerButtons.push(
      <div key={i} className="flex flex-col items-center">
        <RemainingBars remaining={remaining} />
        <button
          className={`${btnBase} ${digitTextClass} font-bold ${stateClass}`}
          disabled={answerDisabled}
          {...tapProps(() => onAnswer(i), answerDisabled)}
        >
          {i}
        </button>
      </div>,
    );
  }
  answerButtons.push(
    <div key="clear" className="flex flex-col justify-end">
      <button
        className={`${btnBase} ${clearTextClass} ${answerDisabled ? btnDisabled : `${btnDefault} text-red-600 dark:text-red-400`}`}
        disabled={answerDisabled}
        {...tapProps(onClearAnswer, answerDisabled)}
      >
        ×
      </button>
    </div>,
  );

  // Memo row
  const memoButtons: React.ReactNode[] = [];
  for (let i = 1; i <= n; i++) {
    const isActive = !memoDisabled && (currentCandidates?.has(i) ?? false);
    memoButtons.push(
      <button
        key={i}
        className={`${btnBase} ${digitTextClass} font-light ${memoDisabled ? btnDisabled : isActive ? btnActiveCandidate : `${btnDefault} text-gray-500 dark:text-slate-400`}`}
        disabled={memoDisabled}
        {...tapProps(() => onToggleCandidate(i), memoDisabled)}
      >
        {i}
      </button>,
    );
  }
  memoButtons.push(
    <button
      key="clear"
      className={`${btnBase} flex items-center justify-center ${memoDisabled ? `${btnDisabled}` : `${btnDefault} text-red-600 dark:text-red-400`}`}
      disabled={memoDisabled}
      aria-label="Clear candidates"
      {...tapProps(onClearCandidates, memoDisabled)}
    >
      <EraserIcon />
    </button>,
  );

  return (
    <div className="flex flex-col items-center gap-1.5 mt-8 mb-5">
      <div
        className={`flex ${rowGapClass} items-end justify-center flex-wrap max-w-[90vw]`}
      >
        {answerButtons}
      </div>
      <div
        className={`flex ${rowGapClass} justify-center flex-wrap max-w-[90vw]`}
      >
        {memoButtons}
      </div>
    </div>
  );
}
