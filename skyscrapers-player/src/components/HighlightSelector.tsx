import { useEffect, useRef, useState } from "react";

interface HighlightSelectorProps {
  n: number;
  value: number | null;
  onChange: (next: number | null) => void;
}

function ChevronIcon({ open }: { open: boolean }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={`w-4 h-4 transition-transform duration-100 ${open ? "rotate-180" : ""}`}
      aria-hidden="true"
    >
      <path d="M6 9l6 6 6-6" />
    </svg>
  );
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

function HighlighterIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className="w-4 h-4 shrink-0"
      aria-hidden="true"
    >
      <path d="M3 19h4l10.5-10.5a2.83 2.83 0 1 0-4-4L3 15v4" />
      <path d="M12.5 5.5l4 4" />
      <path d="M4.5 13.5l4 4" />
    </svg>
  );
}

/**
 * Dedicated highlight picker, decoupled from cell selection and value entry.
 *
 * The number pad's answer row is value-entry only, so the sole way to highlight
 * a digit board-wide — including digits not yet placed anywhere — is this
 * control. Keeping highlight on its own affordance removes the old overload
 * where the same number button meant "place" or "highlight" depending on
 * whether a cell happened to be selected, which could misfire as a wrong entry.
 */
export function HighlightSelector({
  n,
  value,
  onChange,
}: HighlightSelectorProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const active = value !== null;

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (e: PointerEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        // Close only the popover; don't let Player's window-level Escape also
        // clear the active highlight.
        e.stopPropagation();
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

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

  const select = (v: number) => {
    onChange(v === value ? null : v);
    setOpen(false);
  };

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        className={`${pillBase} ${active ? pillActive : pillDefault}`}
        onClick={() => setOpen((o) => !o)}
        aria-haspopup="true"
        aria-expanded={open}
        aria-label={
          active
            ? `Highlighting ${value}. Change highlight`
            : "Highlight a number"
        }
      >
        <HighlighterIcon />
        {active ? (
          <span className="font-medium tabular-nums">{value}</span>
        ) : (
          <span className="hidden sm:inline">Highlight</span>
        )}
        <ChevronIcon open={open} />
      </button>

      {open && (
        <div
          role="menu"
          className="absolute right-0 z-10 mt-1 p-2 bg-white dark:bg-slate-800 border border-gray-300 dark:border-slate-600 rounded-md shadow-lg flex flex-wrap gap-1.5 w-max max-w-[calc(100vw-2.5rem)]"
        >
          {Array.from({ length: n }, (_, i) => {
            const v = i + 1;
            const isActive = v === value;
            return (
              <button
                key={v}
                type="button"
                className={`${chipBase} ${isActive ? chipActive : chipDefault}`}
                onClick={() => select(v)}
                aria-pressed={isActive}
              >
                {v}
              </button>
            );
          })}
          <button
            type="button"
            className={`${chipBase} flex items-center justify-center ${active ? `${chipDefault} text-red-600 dark:text-red-400` : "border-gray-300 dark:border-slate-700 text-gray-300 dark:text-slate-600 cursor-not-allowed"}`}
            onClick={() => {
              onChange(null);
              setOpen(false);
            }}
            disabled={!active}
            aria-label="Clear highlight"
          >
            <ClearIcon />
          </button>
        </div>
      )}
    </div>
  );
}
