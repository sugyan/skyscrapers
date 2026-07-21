import { useEffect, useRef, useState } from "react";
import { HighlightPill, HighlightPanel } from "./HighlightSelector";

interface GameControlsProps {
  n: number;
  highlightValue: number | null;
  onHighlightChange: (next: number | null) => void;
  canUndo: boolean;
  onUndo: () => void;
  onReset: () => void;
  onHint: () => void;
  onCheck: () => void;
  onFillCandidates: () => void;
}

export function GameControls({
  n,
  highlightValue,
  onHighlightChange,
  canUndo,
  onUndo,
  onReset,
  onHint,
  onCheck,
  onFillCandidates,
}: GameControlsProps) {
  const [highlightOpen, setHighlightOpen] = useState(false);
  const pillRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!highlightOpen) return;
    const onPointerDown = (e: PointerEvent) => {
      const t = e.target as Node;
      if (!pillRef.current?.contains(t) && !panelRef.current?.contains(t)) {
        setHighlightOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        // Close only the panel; stop Player's window-level Escape from also
        // deselecting the current cell or clearing the active highlight.
        e.stopPropagation();
        setHighlightOpen(false);
        return;
      }
      // While focus is in the picker, keep keys Player handles globally
      // (digits, Space, Backspace/Delete, arrows, Tab) from bubbling to its
      // window-level handler and mutating the board.
      const a = document.activeElement;
      if (pillRef.current?.contains(a) || panelRef.current?.contains(a)) {
        e.stopPropagation();
      }
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [highlightOpen]);

  const handleSelect = (v: number) => {
    onHighlightChange(v === highlightValue ? null : v);
    setHighlightOpen(false);
  };
  const handleClear = () => {
    onHighlightChange(null);
    setHighlightOpen(false);
  };

  const primaryBtn =
    "px-4 py-2 text-base border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed touch-manipulation select-none";
  const secondaryBtn =
    "px-3 py-1.5 text-sm border border-gray-400 dark:border-slate-600 rounded-md bg-white dark:bg-slate-800 cursor-pointer hover:bg-gray-200 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed touch-manipulation select-none";

  return (
    <div className="flex flex-col gap-2 my-4 w-full max-w-md">
      <div className="grid grid-cols-3 gap-2">
        <button
          className={primaryBtn}
          onClick={onUndo}
          disabled={!canUndo}
          aria-label="Undo"
        >
          Undo
        </button>
        <button className={primaryBtn} onClick={onCheck}>
          Check
        </button>
        <div className="flex" ref={pillRef}>
          <HighlightPill
            active={highlightValue !== null}
            value={highlightValue}
            open={highlightOpen}
            onToggle={() => setHighlightOpen((o) => !o)}
          />
        </div>
      </div>
      {highlightOpen && (
        <div ref={panelRef}>
          <HighlightPanel
            n={n}
            value={highlightValue}
            onSelect={handleSelect}
            onClear={handleClear}
          />
        </div>
      )}
      <div className="grid grid-cols-3 gap-2">
        <button className={secondaryBtn} onClick={onHint}>
          Hint
        </button>
        <button className={secondaryBtn} onClick={onFillCandidates}>
          Fill memo
        </button>
        <button className={secondaryBtn} onClick={onReset}>
          Reset
        </button>
      </div>
    </div>
  );
}
