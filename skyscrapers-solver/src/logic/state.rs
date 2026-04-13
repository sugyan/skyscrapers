use skyscrapers_core::Puzzle;

use crate::candidates::Candidates;

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
    /// Build initial state from a puzzle. Returns None if contradictory.
    pub fn new(puzzle: &Puzzle) -> Option<Self> {
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

        // Apply clue-based pruning before board values
        if !super::techniques::clue_pruning::apply(&mut state) {
            return None;
        }

        // Assign any singletons created by clue pruning
        for idx in 0..n * n {
            if state.grid[idx].is_none() {
                if let Some(v) = state.candidates[idx].singleton() {
                    let r = idx / n;
                    let c = idx % n;
                    if !state.assign(r, c, v) {
                        return None;
                    }
                }
            }
        }

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

        Some(state)
    }

    /// Assign value `v` to cell (r, c) and propagate constraints.
    /// Returns false if contradiction detected.
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
    /// If this leaves a naked single, assign it.
    /// Returns false if contradiction (empty candidate set).
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

        // Naked single propagation
        // TODO: This implicit assignment means Step.actions may not fully describe
        // the state transition — a technique reporting only eliminations can also
        // cause unreported placements. Consider separating propagation from the
        // low-level state mutation, or recording the implied placements.
        if let Some(sv) = new.singleton() {
            if !self.assign(r, c, sv) {
                return false;
            }
        }

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
        let state = SolveState::new(&puzzle).unwrap();
        assert_eq!(state.n, 4);
        assert!(state.grid.iter().all(|c| c.is_none()));
        assert!(state.candidates.iter().all(|c| c.count() == 4));
    }

    #[test]
    fn assign_propagates() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

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
