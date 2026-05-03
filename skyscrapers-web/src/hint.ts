import type { CluePosition, HintResult, Line, Technique } from "./wasm";
import type { BoardCell } from "./types";

export const TECHNIQUE_LABELS: Record<Technique, string> = {
  "naked-singles": "Naked Single",
  "hidden-singles": "Hidden Single",
  "clue-pruning": "Clue Pruning",
  "visibility-analysis": "Visibility Analysis",
  "naked-sets": "Naked Set",
  "x-wing": "X-Wing",
  "als-xz": "ALS-XZ",
  "permutation-enumeration": "Permutation Enumeration",
  "dual-clue-permutation": "Dual-Clue Permutation",
  "simple-forcing-chain": "Simple Forcing Chain",
  "full-forcing-chain": "Full Forcing Chain",
};

export function lineLabel(line: Line): string {
  return "row" in line ? `row ${line.row + 1}` : `column ${line.col + 1}`;
}

export function cluePositionLabel(clue: CluePosition): string {
  if ("top" in clue) return `top clue of column ${clue.top + 1}`;
  if ("bottom" in clue) return `bottom clue of column ${clue.bottom + 1}`;
  if ("left" in clue) return `left clue of row ${clue.left + 1}`;
  return `right clue of row ${clue.right + 1}`;
}

export function cellLabel(row: number, col: number): string {
  return `R${row + 1}C${col + 1}`;
}

/** Cells that the hint reasoning touches, gathered from actions + reason. */
export function relevantCells(hint: HintResult): [number, number][] {
  const seen = new Set<string>();
  const out: [number, number][] = [];
  const push = (r: number, c: number) => {
    const key = `${r},${c}`;
    if (!seen.has(key)) {
      seen.add(key);
      out.push([r, c]);
    }
  };

  for (const action of hint.step.actions) {
    push(action.row, action.col);
  }

  const reason = hint.step.reason;
  switch (reason.kind) {
    case "single-candidate":
      push(reason.row, reason.col);
      break;
    case "set-in-line":
      reason.cells.forEach(([r, c]) => push(r, c));
      break;
    case "als-xz-elimination":
      reason.als_a.forEach(([r, c]) => push(r, c));
      reason.als_b.forEach(([r, c]) => push(r, c));
      break;
    case "forcing-chain-elimination":
      push(reason.assumed_cell[0], reason.assumed_cell[1]);
      break;
    default:
      break;
  }
  return out;
}

/** Lines (rows/cols) the hint references, used for grid highlighting. */
export function relevantLines(hint: HintResult): Line[] {
  const reason = hint.step.reason;
  switch (reason.kind) {
    case "unique-in-line":
    case "set-in-line":
    case "permutation-elimination":
    case "dual-clue-permutation-elimination":
    case "visibility-forcing":
      return [reason.line];
    case "fish-pattern":
      return [...reason.base_lines, ...reason.cover_lines];
    default:
      return [];
  }
}

/** Clue positions the hint references, used for grid highlighting. */
export function relevantClues(hint: HintResult): CluePosition[] {
  const reason = hint.step.reason;
  switch (reason.kind) {
    case "permutation-elimination":
    case "visibility-forcing":
    case "initial-clue-constraint":
      return [reason.clue];
    case "dual-clue-permutation-elimination":
      return [reason.clue_a, reason.clue_b];
    default:
      return [];
  }
}

/** Plain-language summary of why the solver took this step. */
export function reasonText(hint: HintResult): string {
  const reason = hint.step.reason;
  switch (reason.kind) {
    case "single-candidate":
      return `${cellLabel(reason.row, reason.col)} has only one remaining candidate.`;
    case "unique-in-line":
      return `${reason.value} can only go in one cell of ${lineLabel(reason.line)}.`;
    case "set-in-line": {
      const cells = reason.cells.map(([r, c]) => cellLabel(r, c)).join(", ");
      return `In ${lineLabel(reason.line)}, the values {${reason.values.join(", ")}} are confined to ${cells}.`;
    }
    case "fish-pattern":
      return `Fish pattern on value ${reason.value}: candidates in ${reason.base_lines.map(lineLabel).join(", ")} are covered by ${reason.cover_lines.map(lineLabel).join(", ")}.`;
    case "permutation-elimination":
      return `Permutation enumeration on ${lineLabel(reason.line)} (${cluePositionLabel(reason.clue)}) rules out the eliminated candidates.`;
    case "dual-clue-permutation-elimination":
      return `Combining ${cluePositionLabel(reason.clue_a)} and ${cluePositionLabel(reason.clue_b)} on ${lineLabel(reason.line)} rules out the eliminated candidates.`;
    case "als-xz-elimination":
      return `ALS-XZ on value ${reason.rcc_value} eliminates ${reason.eliminated_value} from cells seeing both sets.`;
    case "forcing-chain-elimination":
      return `Assuming ${reason.assumed_value} at ${cellLabel(reason.assumed_cell[0], reason.assumed_cell[1])} leads to a contradiction, so it can be eliminated.`;
    case "initial-clue-constraint":
      return `The ${cluePositionLabel(reason.clue)} forces this elimination from the start.`;
    case "visibility-forcing":
      return `Visibility analysis on ${lineLabel(reason.line)} (${cluePositionLabel(reason.clue)}) forces this constraint.`;
  }
}

export interface CellCandidateDiff {
  row: number;
  col: number;
  expected: number[];
  /** Candidates user has but the solver does not — should be removed. */
  extra: number[];
  /** Candidates the solver has but the user does not — should be added. */
  missing: number[];
  userEmpty: boolean;
  /** True iff the cell is already confirmed in the user's board. */
  confirmed: boolean;
}

/** Compute the per-cell candidate diff for cells the hint touches. */
export function candidateDiffs(
  hint: HintResult,
  board: BoardCell[][],
): CellCandidateDiff[] {
  return relevantCells(hint).map(([r, c]) => {
    const expected = hint.candidates[r][c];
    const cell = board[r][c];
    const userSet = cell.candidates;
    const expectedSet = new Set(expected);
    const extra = [...userSet].filter((v) => !expectedSet.has(v)).sort();
    const missing = expected.filter((v) => !userSet.has(v));
    return {
      row: r,
      col: c,
      expected,
      extra,
      missing,
      userEmpty: userSet.size === 0 && cell.value === null,
      confirmed: cell.value !== null,
    };
  });
}

export function hasCandidateMismatch(diffs: CellCandidateDiff[]): boolean {
  return diffs.some(
    (d) => !d.confirmed && (d.extra.length > 0 || d.missing.length > 0),
  );
}
