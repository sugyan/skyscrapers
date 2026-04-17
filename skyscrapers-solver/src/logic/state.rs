use skyscrapers_core::Puzzle;

use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};

/// Mutable solving state for the logic solver.
///
/// Similar to the backtracking solver's SolveState but without undo logs
/// (the logic solver never speculatively assigns).
#[derive(Clone)]
pub(crate) struct SolveState {
    pub n: usize,
    pub grid: Vec<Option<u8>>,
    pub candidates: Vec<Candidates>,
    pub top: Vec<Option<u8>>,
    pub bottom: Vec<Option<u8>>,
    pub left: Vec<Option<u8>>,
    pub right: Vec<Option<u8>>,
}

impl SolveState {
    /// Build initial state from a puzzle.
    ///
    /// Returns `None` if contradictory, else the state paired with any [`Step`]s
    /// produced by the one-shot clue-based pruning performed during init.
    /// Callers typically prepend these Steps to the solve trace so downstream
    /// naked-singles like "R1C1 = 1" have a visible antecedent.
    pub fn new(puzzle: &Puzzle) -> Option<(Self, Vec<Step>)> {
        let n = puzzle.board.n();
        if n == 0 || n > 9 || puzzle.clues.n() != n {
            return None;
        }
        let clues = &puzzle.clues;

        let mut state = Self {
            n,
            grid: vec![None; n * n],
            candidates: vec![Candidates::all(n as u8); n * n],
            top: (0..n).map(|i| clues.top(i)).collect(),
            bottom: (0..n).map(|i| clues.bottom(i)).collect(),
            left: (0..n).map(|i| clues.left(i)).collect(),
            right: (0..n).map(|i| clues.right(i)).collect(),
        };

        // Apply clue-based pruning before board values. This may leave some
        // cells with a singleton candidate set (clue=1 or clue=n cases).
        let mut init_steps = super::techniques::clue_pruning::apply(&mut state)?;

        // Assign any singletons created by clue pruning. We attribute the
        // resulting placements back to the clue Step(s) that caused them so
        // the trace shows e.g. "[CluePruning] Left=5 @ Row 0: R0C0=1, ..."
        // instead of a bare elimination list.
        let mut placement_cells: Vec<(usize, usize, u8)> = Vec::new();
        for idx in 0..n * n {
            if state.grid[idx].is_none() {
                if let Some(v) = state.candidates[idx].singleton() {
                    let r = idx / n;
                    let c = idx % n;
                    if !state.assign(r, c, v) {
                        return None;
                    }
                    placement_cells.push((r, c, v));
                }
            }
        }
        attribute_placements_to_steps(&mut init_steps, &placement_cells, n);

        // Place board values with constraint propagation
        for r in 0..n {
            for c in 0..n {
                if let Some(v) = puzzle.board.get(r, c) {
                    if !state.assign(r, c, v) {
                        return None;
                    }
                }
            }
        }

        Some((state, init_steps))
    }

    /// Assign value `v` to cell (r, c) and propagate Latin-square constraints
    /// (eliminate `v` from all peers in the same row and column).
    ///
    /// Returns false if this assignment contradicts the current state (the
    /// cell is already set to a different value, `v` is not a candidate, or
    /// propagation empties another cell's candidate set).
    pub fn assign(&mut self, r: usize, c: usize, v: u8) -> bool {
        let idx = r * self.n + c;

        // Already assigned
        if let Some(existing) = self.grid[idx] {
            return existing == v;
        }
        // Value not in candidate set
        if !self.candidates[idx].contains(v) {
            return false;
        }

        self.grid[idx] = Some(v);
        self.candidates[idx] = Candidates::single(v);

        // Eliminate v from peers (same row and column)
        for j in 0..self.n {
            if j != c && !self.eliminate(r, j, v) {
                return false;
            }
            if j != r && !self.eliminate(j, c, v) {
                return false;
            }
        }

        true
    }

    /// Remove value `v` from candidates of cell (r, c).
    /// Returns false if contradiction (empty candidate set).
    ///
    /// Note: if this leaves a cell with a single candidate, the cell is **not**
    /// auto-assigned here. The outer solver loop will pick up the naked single
    /// on the next iteration via `NakedSingles`, so each placement is recorded
    /// as its own explicit `Step`.
    pub fn eliminate(&mut self, r: usize, c: usize, v: u8) -> bool {
        let idx = r * self.n + c;
        if self.grid[idx].is_some() || !self.candidates[idx].contains(v) {
            return true; // already assigned or not a candidate
        }

        let new = self.candidates[idx].remove(v);
        if new.is_empty() {
            return false;
        }
        self.candidates[idx] = new;

        true
    }

    /// Returns true if all cells are assigned.
    pub fn is_complete(&self) -> bool {
        self.grid.iter().all(|c| c.is_some())
    }

    /// Convert the grid to a Solution. Panics if not complete.
    pub fn to_solution(&self) -> skyscrapers_core::Solution {
        let cells: Vec<Vec<u8>> = self
            .grid
            .chunks(self.n)
            .map(|row| row.iter().map(|v| v.unwrap()).collect())
            .collect();
        skyscrapers_core::Solution::new(self.n, cells)
    }

    /// Verify all clue constraints against the completed grid.
    pub fn verify_clues(&self) -> bool {
        for i in 0..self.n {
            if let Some(expected) = self.top[i] {
                if self.count_visible_col(i, true) != expected {
                    return false;
                }
            }
            if let Some(expected) = self.bottom[i] {
                if self.count_visible_col(i, false) != expected {
                    return false;
                }
            }
            if let Some(expected) = self.left[i] {
                if self.count_visible_row(i, true) != expected {
                    return false;
                }
            }
            if let Some(expected) = self.right[i] {
                if self.count_visible_row(i, false) != expected {
                    return false;
                }
            }
        }
        true
    }

    fn count_visible_col(&self, col: usize, top_to_bottom: bool) -> u8 {
        let mut max = 0u8;
        let mut count = 0u8;
        for r in 0..self.n {
            let row = if top_to_bottom { r } else { self.n - 1 - r };
            let h = self.grid[row * self.n + col].unwrap();
            if h > max {
                count += 1;
                max = h;
            }
        }
        count
    }

    fn count_visible_row(&self, row: usize, left_to_right: bool) -> u8 {
        let mut max = 0u8;
        let mut count = 0u8;
        for c in 0..self.n {
            let col = if left_to_right { c } else { self.n - 1 - c };
            let h = self.grid[row * self.n + col].unwrap();
            if h > max {
                count += 1;
                max = h;
            }
        }
        count
    }

    /// Index helper: (r, c) -> flat index
    #[inline]
    pub fn idx(&self, r: usize, c: usize) -> usize {
        r * self.n + c
    }

    /// Iterate every row/column that has a clue set on at least one side.
    ///
    /// `indices` are in viewing order for the given clue (e.g., Right-clue lines
    /// yield right-to-left indices). Used by single-clue permutation enumeration.
    pub(crate) fn clued_lines(&self) -> Vec<CluedLine> {
        let n = self.n;
        let mut out = Vec::with_capacity(4 * n);
        for i in 0..n {
            if let Some(expected) = self.left[i] {
                out.push(CluedLine {
                    indices: (0..n).map(|c| i * n + c).collect(),
                    expected,
                    line: Line::Row(i),
                    clue_pos: CluePosition::Left(i),
                });
            }
            if let Some(expected) = self.right[i] {
                out.push(CluedLine {
                    indices: (0..n).rev().map(|c| i * n + c).collect(),
                    expected,
                    line: Line::Row(i),
                    clue_pos: CluePosition::Right(i),
                });
            }
            if let Some(expected) = self.top[i] {
                out.push(CluedLine {
                    indices: (0..n).map(|r| r * n + i).collect(),
                    expected,
                    line: Line::Col(i),
                    clue_pos: CluePosition::Top(i),
                });
            }
            if let Some(expected) = self.bottom[i] {
                out.push(CluedLine {
                    indices: (0..n).rev().map(|r| r * n + i).collect(),
                    expected,
                    line: Line::Col(i),
                    clue_pos: CluePosition::Bottom(i),
                });
            }
        }
        out
    }

    /// Iterate every row/column that has both opposing clues set.
    ///
    /// `indices` are in natural order (left-to-right for rows, top-to-bottom for
    /// columns); `expected_fwd` is the Left/Top clue, `expected_rev` is the
    /// Right/Bottom clue. Used by dual-clue permutation enumeration.
    pub(crate) fn dual_clued_lines(&self) -> Vec<DualCluedLine> {
        let n = self.n;
        let mut out = Vec::with_capacity(2 * n);
        for i in 0..n {
            if let (Some(fwd), Some(rev)) = (self.left[i], self.right[i]) {
                out.push(DualCluedLine {
                    indices: (0..n).map(|c| i * n + c).collect(),
                    expected_fwd: fwd,
                    expected_rev: rev,
                    line: Line::Row(i),
                    clue_fwd: CluePosition::Left(i),
                    clue_rev: CluePosition::Right(i),
                });
            }
            if let (Some(fwd), Some(rev)) = (self.top[i], self.bottom[i]) {
                out.push(DualCluedLine {
                    indices: (0..n).map(|r| r * n + i).collect(),
                    expected_fwd: fwd,
                    expected_rev: rev,
                    line: Line::Col(i),
                    clue_fwd: CluePosition::Top(i),
                    clue_rev: CluePosition::Bottom(i),
                });
            }
        }
        out
    }
}

/// Attach `Action::Place` entries to whichever clue-pruning `Step` first
/// touched the given cell. If no existing Step mentions the cell (rare — would
/// require a singleton from an empty init), the placements are folded into
/// the last Step as a best-effort.
fn attribute_placements_to_steps(
    steps: &mut Vec<Step>,
    placements: &[(usize, usize, u8)],
    n: usize,
) {
    if placements.is_empty() {
        return;
    }
    for &(r, c, v) in placements {
        let cell_idx = r * n + c;
        let mut attached = false;
        for step in steps.iter_mut() {
            if step_touches_cell(step, cell_idx, n) {
                step.actions.push(Action::Place {
                    row: r,
                    col: c,
                    value: v,
                });
                attached = true;
                break;
            }
        }
        if !attached {
            // Fallback: emit a standalone placement step. This handles the
            // exotic case where the singleton emerged from cross-clue
            // elimination but the original Steps were coalesced away.
            steps.push(Step {
                technique: Technique::CluePruning,
                actions: vec![Action::Place {
                    row: r,
                    col: c,
                    value: v,
                }],
                reason: Reason::InitialClueConstraint {
                    clue: CluePosition::Top(c),
                },
            });
        }
    }
}

fn step_touches_cell(step: &Step, cell_idx: usize, n: usize) -> bool {
    step.actions.iter().any(|a| match a {
        Action::Eliminate { row, col, .. } | Action::Place { row, col, .. } => {
            row * n + col == cell_idx
        }
    })
}

/// A row or column with a single clue, prepared for permutation enumeration.
pub(crate) struct CluedLine {
    pub indices: Vec<usize>,
    pub expected: u8,
    pub line: Line,
    pub clue_pos: CluePosition,
}

/// A row or column with both opposing clues, prepared for dual-clue enumeration.
pub(crate) struct DualCluedLine {
    pub indices: Vec<usize>,
    pub expected_fwd: u8,
    pub expected_rev: u8,
    pub line: Line,
    pub clue_fwd: CluePosition,
    pub clue_rev: CluePosition,
}

/// Two cells "see" each other iff they share a row or column.
///
/// Used by peer-based techniques (XY-Wing, W-Wing, ALS-XZ) to check whether
/// an elimination target is linked to a pattern's anchor cells.
#[inline]
pub(crate) fn sees(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 == b.0 || a.1 == b.1
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn new_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (state, init_steps) = SolveState::new(&puzzle).unwrap();
        assert_eq!(state.n, 4);
        assert!(state.grid.iter().all(|c| c.is_none()));
        assert!(state.candidates.iter().all(|c| c.count() == 4));
        assert!(init_steps.is_empty());
    }

    #[test]
    fn assign_propagates() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        assert!(state.assign(0, 0, 3));
        // Row 0 peers should not have 3
        for c in 1..4 {
            assert!(!state.candidates[state.idx(0, c)].contains(3));
        }
        // Col 0 peers should not have 3
        for r in 1..4 {
            assert!(!state.candidates[state.idx(r, 0)].contains(3));
        }
    }

    #[test]
    fn contradictory_assign() {
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(1)); // duplicate in row
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        assert!(SolveState::new(&puzzle).is_none());
    }
}
