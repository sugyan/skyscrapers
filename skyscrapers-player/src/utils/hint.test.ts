import { describe, it, expect } from "vitest";
import {
  TECHNIQUE_DESCRIPTIONS,
  TECHNIQUE_LABELS,
  candidateDiffs,
  cellLabel,
  cluePositionLabel,
  hasCandidateMismatch,
  lineLabel,
  reasonText,
  relevantCells,
  relevantClues,
  relevantLines,
} from "./hint";
import type { HintResult } from "../engine/types";
import type { BoardCell } from "../state/types";

function cell(
  value: number | null,
  candidates: number[] = [],
  given = false,
): BoardCell {
  return {
    value,
    given,
    candidates: new Set(candidates),
  };
}

function emptyBoard(n: number): BoardCell[][] {
  return Array.from({ length: n }, () =>
    Array.from({ length: n }, () => cell(null)),
  );
}

function placeHint(
  row: number,
  col: number,
  value: number,
  candidates: number[][][],
): HintResult {
  return {
    step: {
      technique: "naked-singles",
      actions: [{ kind: "place", row, col, value }],
      reason: { kind: "single-candidate", row, col },
    },
    candidates,
  };
}

function eliminateHint(
  row: number,
  col: number,
  value: number,
  line: { row: number } | { col: number },
  candidates: number[][][],
): HintResult {
  return {
    step: {
      technique: "hidden-singles",
      actions: [{ kind: "eliminate", row, col, value }],
      reason: { kind: "unique-in-line", line, value },
    },
    candidates,
  };
}

describe("labels", () => {
  it("covers every Technique key", () => {
    for (const value of Object.values(TECHNIQUE_LABELS)) {
      expect(value.length).toBeGreaterThan(0);
    }
  });

  it("has a non-empty description for every technique", () => {
    const labelKeys = Object.keys(TECHNIQUE_LABELS);
    const descKeys = Object.keys(TECHNIQUE_DESCRIPTIONS);
    expect(descKeys.sort()).toEqual(labelKeys.sort());
    for (const value of Object.values(TECHNIQUE_DESCRIPTIONS)) {
      expect(value.length).toBeGreaterThan(0);
    }
  });

  it("formats cell coordinates 1-indexed", () => {
    expect(cellLabel(0, 0)).toBe("R1C1");
    expect(cellLabel(2, 4)).toBe("R3C5");
  });

  it("formats line labels", () => {
    expect(lineLabel({ row: 0 })).toBe("row 1");
    expect(lineLabel({ col: 3 })).toBe("column 4");
  });

  it("formats clue position labels", () => {
    expect(cluePositionLabel({ top: 0 })).toBe("top clue of column 1");
    expect(cluePositionLabel({ bottom: 1 })).toBe("bottom clue of column 2");
    expect(cluePositionLabel({ left: 2 })).toBe("left clue of row 3");
    expect(cluePositionLabel({ right: 3 })).toBe("right clue of row 4");
  });
});

describe("relevantCells", () => {
  const candidates = [
    [[], []],
    [[], []],
  ] as number[][][];

  it("returns the action cell for a Place + SingleCandidate hint", () => {
    const hint = placeHint(0, 1, 5, candidates);
    expect(relevantCells(hint)).toEqual([[0, 1]]);
  });

  it("includes set-in-line cells alongside the action cell", () => {
    const hint: HintResult = {
      step: {
        technique: "naked-sets",
        actions: [{ kind: "eliminate", row: 0, col: 0, value: 4 }],
        reason: {
          kind: "set-in-line",
          line: { row: 0 },
          cells: [
            [0, 1],
            [0, 2],
          ],
          values: [1, 2],
        },
      },
      candidates,
    };
    expect(relevantCells(hint)).toEqual([
      [0, 0],
      [0, 1],
      [0, 2],
    ]);
  });

  it("returns chain cells for xy-chain elimination", () => {
    const hint: HintResult = {
      step: {
        technique: "xy-chain",
        actions: [{ kind: "eliminate", row: 3, col: 0, value: 3 }],
        reason: {
          kind: "xy-chain-elimination",
          chain: [
            [0, 0],
            [2, 0],
            [2, 2],
            [3, 2],
          ],
          eliminated_value: 3,
        },
      },
      candidates,
    };
    expect(relevantCells(hint)).toEqual([
      [3, 0],
      [0, 0],
      [2, 0],
      [2, 2],
      [3, 2],
    ]);
  });

  it("deduplicates cells appearing in both actions and reason", () => {
    const hint: HintResult = {
      step: {
        technique: "als-xz",
        actions: [{ kind: "eliminate", row: 1, col: 1, value: 3 }],
        reason: {
          kind: "als-xz-elimination",
          als_a: [
            [0, 0],
            [0, 1],
          ],
          als_b: [[1, 1]],
          rcc_value: 2,
          eliminated_value: 3,
        },
      },
      candidates,
    };
    expect(relevantCells(hint)).toEqual([
      [1, 1],
      [0, 0],
      [0, 1],
    ]);
  });
});

describe("relevantLines and relevantClues", () => {
  const candidates = [[[]]] as number[][][];

  it("returns base + cover lines for fish patterns", () => {
    const hint: HintResult = {
      step: {
        technique: "x-wing",
        actions: [],
        reason: {
          kind: "fish-pattern",
          value: 4,
          base_lines: [{ row: 0 }, { row: 2 }],
          cover_lines: [{ col: 1 }, { col: 3 }],
        },
      },
      candidates,
    };
    expect(relevantLines(hint)).toEqual([
      { row: 0 },
      { row: 2 },
      { col: 1 },
      { col: 3 },
    ]);
    expect(relevantClues(hint)).toEqual([]);
  });

  it("returns the clue from initial-clue-constraint reasons", () => {
    const hint: HintResult = {
      step: {
        technique: "clue-pruning",
        actions: [],
        reason: { kind: "initial-clue-constraint", clue: { top: 2 } },
      },
      candidates,
    };
    expect(relevantClues(hint)).toEqual([{ top: 2 }]);
    expect(relevantLines(hint)).toEqual([]);
  });
});

describe("reasonText", () => {
  const candidates = [[[]]] as number[][][];

  it("describes a naked single", () => {
    expect(reasonText(placeHint(1, 2, 4, candidates))).toBe(
      "R2C3 has only one remaining candidate.",
    );
  });

  it("describes a hidden single in a line", () => {
    expect(reasonText(eliminateHint(0, 0, 5, { row: 0 }, candidates))).toBe(
      "5 can only go in one cell of row 1.",
    );
  });

  it("describes a fish pattern", () => {
    const hint: HintResult = {
      step: {
        technique: "x-wing",
        actions: [],
        reason: {
          kind: "fish-pattern",
          value: 3,
          base_lines: [{ row: 0 }, { row: 2 }],
          cover_lines: [{ col: 1 }, { col: 4 }],
        },
      },
      candidates,
    };
    expect(reasonText(hint)).toContain("Fish pattern on value 3");
    expect(reasonText(hint)).toContain("row 1, row 3");
    expect(reasonText(hint)).toContain("column 2, column 5");
  });

  it("names both ALS groups and the link/eliminated values", () => {
    const hint: HintResult = {
      step: {
        technique: "als-xz",
        actions: [{ kind: "eliminate", row: 1, col: 4, value: 4 }],
        reason: {
          kind: "als-xz-elimination",
          als_a: [
            [1, 0],
            [1, 1],
            [1, 3],
          ],
          als_b: [
            [2, 0],
            [2, 1],
            [2, 3],
            [2, 4],
          ],
          rcc_value: 5,
          eliminated_value: 4,
        },
      },
      candidates,
    };
    expect(reasonText(hint)).toBe(
      "ALS-XZ: groups {R2C1, R2C2, R2C4} and {R3C1, R3C2, R3C4, R3C5} " +
        "link through 5, forcing 4 into one of them — so 4 is removed " +
        "from cells seeing both.",
    );
  });
});

describe("candidateDiffs and hasCandidateMismatch", () => {
  it("flags missing and extra candidates against the solver expectation", () => {
    const board = emptyBoard(4);
    board[0][0] = cell(null, [3, 4]); // expected {1,2,4} → missing {1,2}, extra {3}
    const candidates: number[][][] = [
      [[1, 2, 4], [], [], []],
      [[], [], [], []],
      [[], [], [], []],
      [[], [], [], []],
    ];
    const hint = placeHint(0, 0, 1, candidates);
    const diffs = candidateDiffs(hint, board);
    expect(diffs).toHaveLength(1);
    expect(diffs[0]).toMatchObject({
      row: 0,
      col: 0,
      expected: [1, 2, 4],
      missing: [1, 2],
      extra: [3],
      userEmpty: false,
      confirmed: false,
    });
    expect(hasCandidateMismatch(diffs)).toBe(true);
  });

  it("treats a confirmed cell as having no diff to surface", () => {
    const board = emptyBoard(4);
    board[1][1] = cell(2);
    const candidates: number[][][] = [
      [[], [], [], []],
      [[], [2], [], []],
      [[], [], [], []],
      [[], [], [], []],
    ];
    const hint = placeHint(1, 1, 2, candidates);
    const diffs = candidateDiffs(hint, board);
    expect(diffs[0].confirmed).toBe(true);
    expect(hasCandidateMismatch(diffs)).toBe(false);
  });

  it("marks a cell with no pencil marks as userEmpty without diff signal", () => {
    const board = emptyBoard(4);
    const candidates: number[][][] = [
      [[1, 2, 3], [], [], []],
      [[], [], [], []],
      [[], [], [], []],
      [[], [], [], []],
    ];
    const hint = placeHint(0, 0, 1, candidates);
    const diffs = candidateDiffs(hint, board);
    expect(diffs[0].userEmpty).toBe(true);
    expect(diffs[0].missing).toEqual([1, 2, 3]);
    expect(diffs[0].extra).toEqual([]);
    // Missing-only against an empty cell still counts as a mismatch — the
    // user has nothing yet, so Sync candidates is the right next move.
    expect(hasCandidateMismatch(diffs)).toBe(true);
  });

  it("returns no mismatch when user pencil marks already match expected", () => {
    const board = emptyBoard(4);
    board[0][1] = cell(null, [1, 3]);
    const candidates: number[][][] = [
      [[], [1, 3], [], []],
      [[], [], [], []],
      [[], [], [], []],
      [[], [], [], []],
    ];
    const hint = eliminateHint(0, 1, 2, { row: 0 }, candidates);
    const diffs = candidateDiffs(hint, board);
    expect(diffs[0].extra).toEqual([]);
    expect(diffs[0].missing).toEqual([]);
    expect(hasCandidateMismatch(diffs)).toBe(false);
  });
});
