interface HighlightPillProps {
  active: boolean;
  value: number | null;
  open: boolean;
  onToggle: () => void;
}

interface HighlightPanelProps {
  n: number;
  value: number | null;
  onSelect: (value: number) => void;
  onClear: () => void;
}

function ClearIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="w-4 h-4"
      aria-hidden="true"
    >
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  );
}

const pillBase =
  "flex items-center justify-center gap-1.5 w-full px-3 py-2 text-base border rounded-md transition-colors duration-100 touch-manipulation select-none cursor-pointer";
const pillDefault =
  "border-gray-400 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 text-gray-600 dark:text-slate-300";
const pillActive =
  "border-purple-500 dark:border-purple-400 bg-purple-100 dark:bg-purple-900/50 hover:bg-purple-200 dark:hover:bg-purple-900/70 text-purple-800 dark:text-purple-200";

const chipBase =
  "w-9 h-9 border rounded-md text-base font-medium transition-colors duration-100 touch-manipulation select-none cursor-pointer";
const chipDefault =
  "border-gray-400 dark:border-slate-600 bg-white dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 text-gray-700 dark:text-slate-200";
const chipActive =
  "border-purple-500 dark:border-purple-400 bg-purple-100 dark:bg-purple-900/50 hover:bg-purple-200 dark:hover:bg-purple-900/70 text-purple-800 dark:text-purple-200";

/**
 * The button that opens the highlight picker. Shows "Highlight" when nothing is
 * highlighted and the active number (in purple) otherwise.
 *
 * Highlighting is decoupled from cell selection and value entry: the number
 * pad's answer row is value-entry only, so highlighting a digit board-wide —
 * including digits not yet placed anywhere — goes through this picker (or the
 * keyboard digit shortcut when no cell is selected). Open state and dismissal
 * live in GameControls so the panel can expand full-width in the controls flow
 * (see HighlightPanel) rather than overlaying the rows below.
 */
export function HighlightPill({
  active,
  value,
  open,
  onToggle,
}: HighlightPillProps) {
  return (
    <button
      type="button"
      className={`${pillBase} ${active ? pillActive : pillDefault}`}
      onClick={onToggle}
      aria-expanded={open}
      aria-label={
        active
          ? `Highlighting ${value}. Change highlight`
          : "Highlight a number"
      }
    >
      {active ? (
        <span className="font-medium tabular-nums">{value}</span>
      ) : (
        <span>Highlight</span>
      )}
    </button>
  );
}

/**
 * The 1..n chips (plus clear) for picking the highlighted number. Rendered
 * in-flow by GameControls between the control rows so it pushes the rows below
 * down instead of overlapping them. Re-selecting the active number clears it.
 */
export function HighlightPanel({
  n,
  value,
  onSelect,
  onClear,
}: HighlightPanelProps) {
  const active = value !== null;
  return (
    <div
      role="group"
      aria-label="Highlight a number"
      className="flex flex-wrap justify-center gap-1.5 rounded-md border border-gray-300 dark:border-slate-600 bg-white dark:bg-slate-800 p-2"
    >
      {Array.from({ length: n }, (_, i) => {
        const v = i + 1;
        const isActive = v === value;
        return (
          <button
            key={v}
            type="button"
            className={`${chipBase} ${isActive ? chipActive : chipDefault}`}
            onClick={() => onSelect(v)}
            aria-pressed={isActive}
          >
            {v}
          </button>
        );
      })}
      <button
        type="button"
        className={`${chipBase} flex items-center justify-center ${active ? `${chipDefault} text-red-600 dark:text-red-400` : "border-gray-300 dark:border-slate-700 text-gray-300 dark:text-slate-600 cursor-not-allowed"}`}
        onClick={onClear}
        disabled={!active}
        aria-label="Clear highlight"
      >
        <ClearIcon />
      </button>
    </div>
  );
}
