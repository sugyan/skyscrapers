use skyscrapers_core::{Puzzle, Solution};

use crate::Solver;
use crate::candidates::Candidates;

/// A backtracking solver with constraint propagation.
///
/// Stateless — all mutable state is held in `SolveState` per call.
pub struct BacktrackingSolver;

impl Solver for BacktrackingSolver {
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution> {
        if limit == 0 {
            return Vec::new();
        }
        let n = puzzle.board.n();
        let mut state = match SolveState::new(puzzle) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut solutions = Vec::new();
        state.search(&mut solutions, limit);
        solutions
            .into_iter()
            .map(|grid| Solution::new(n, grid.into_iter().map(|v| v.unwrap()).collect()))
            .collect()
    }
}

struct SolveState {
    n: usize,
    grid: Vec<Option<u8>>,
    candidates: Vec<Candidates>,
    top: Vec<Option<u8>>,
    bottom: Vec<Option<u8>>,
    left: Vec<Option<u8>>,
    right: Vec<Option<u8>>,
}

/// An undo entry recording the previous state at an index.
struct UndoEntry {
    idx: usize,
    prev_candidates: Candidates,
    prev_grid: Option<u8>,
}

impl SolveState {
    /// Builds the initial state from a puzzle. Returns None if contradictory.
    fn new(puzzle: &Puzzle) -> Option<Self> {
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

        // Apply clue-based initial pruning before board values
        if !state.apply_clue_pruning() {
            return None;
        }

        // Place board values (propagating constraints)
        for r in 0..n {
            for c in 0..n {
                if let Some(v) = puzzle.board.get(r, c) {
                    let mut undo = Vec::new();
                    if !state.assign(r * n + c, v, &mut undo) {
                        return None;
                    }
                }
            }
        }

        Some(state)
    }

    /// Apply clue-based pruning to narrow initial candidates.
    fn apply_clue_pruning(&mut self) -> bool {
        let n = self.n as u8;

        for i in 0..self.n {
            // Top clues (looking down column i)
            if let Some(clue) = self.top[i] {
                if clue == 1 {
                    self.candidates[i] = self.candidates[i].intersect(Candidates::single(n));
                } else if clue == n {
                    for r in 0..self.n {
                        self.candidates[r * self.n + i] = self.candidates[r * self.n + i]
                            .intersect(Candidates::single(r as u8 + 1));
                    }
                } else {
                    for r in 0..self.n {
                        let max_val = (n as usize + 1 - clue as usize + r).min(n as usize) as u8;
                        for v in (max_val + 1)..=n {
                            self.candidates[r * self.n + i] =
                                self.candidates[r * self.n + i].remove(v);
                        }
                    }
                }
            }

            // Bottom clues (looking up column i)
            if let Some(clue) = self.bottom[i] {
                if clue == 1 {
                    let idx = (self.n - 1) * self.n + i;
                    self.candidates[idx] = self.candidates[idx].intersect(Candidates::single(n));
                } else if clue == n {
                    for r in 0..self.n {
                        let idx = (self.n - 1 - r) * self.n + i;
                        self.candidates[idx] =
                            self.candidates[idx].intersect(Candidates::single(r as u8 + 1));
                    }
                } else {
                    for r in 0..self.n {
                        let dist = self.n - 1 - r;
                        let max_val = (n as usize + 1 - clue as usize + dist).min(n as usize) as u8;
                        for v in (max_val + 1)..=n {
                            self.candidates[r * self.n + i] =
                                self.candidates[r * self.n + i].remove(v);
                        }
                    }
                }
            }

            // Left clues (looking right along row i)
            if let Some(clue) = self.left[i] {
                if clue == 1 {
                    let idx = i * self.n;
                    self.candidates[idx] = self.candidates[idx].intersect(Candidates::single(n));
                } else if clue == n {
                    for c in 0..self.n {
                        let idx = i * self.n + c;
                        self.candidates[idx] =
                            self.candidates[idx].intersect(Candidates::single(c as u8 + 1));
                    }
                } else {
                    for c in 0..self.n {
                        let max_val = (n as usize + 1 - clue as usize + c).min(n as usize) as u8;
                        for v in (max_val + 1)..=n {
                            self.candidates[i * self.n + c] =
                                self.candidates[i * self.n + c].remove(v);
                        }
                    }
                }
            }

            // Right clues (looking left along row i)
            if let Some(clue) = self.right[i] {
                if clue == 1 {
                    let idx = i * self.n + self.n - 1;
                    self.candidates[idx] = self.candidates[idx].intersect(Candidates::single(n));
                } else if clue == n {
                    for c in 0..self.n {
                        let idx = i * self.n + (self.n - 1 - c);
                        self.candidates[idx] =
                            self.candidates[idx].intersect(Candidates::single(c as u8 + 1));
                    }
                } else {
                    for c in 0..self.n {
                        let dist = self.n - 1 - c;
                        let max_val = (n as usize + 1 - clue as usize + dist).min(n as usize) as u8;
                        for v in (max_val + 1)..=n {
                            self.candidates[i * self.n + c] =
                                self.candidates[i * self.n + c].remove(v);
                        }
                    }
                }
            }
        }

        // Check for empty candidates
        self.candidates.iter().all(|c| !c.is_empty())
    }

    /// Assign value `v` to cell at `idx`, propagate constraints.
    /// Returns false if contradiction detected. Records undo entries.
    fn assign(&mut self, idx: usize, v: u8, undo: &mut Vec<UndoEntry>) -> bool {
        // Already assigned — same value is ok, different value is contradiction
        if let Some(existing) = self.grid[idx] {
            return existing == v;
        }
        // Value not in candidate set — contradiction
        if !self.candidates[idx].contains(v) {
            return false;
        }

        let r = idx / self.n;
        let c = idx % self.n;

        // Record and set this cell
        undo.push(UndoEntry {
            idx,
            prev_candidates: self.candidates[idx],
            prev_grid: self.grid[idx],
        });
        self.grid[idx] = Some(v);
        self.candidates[idx] = Candidates::single(v);

        // Eliminate v from same row and column
        let mut propagation_queue = Vec::new();

        for j in 0..self.n {
            // Same row, different column
            if j != c {
                let peer = r * self.n + j;
                if self.grid[peer].is_none() && self.candidates[peer].contains(v) {
                    let prev = self.candidates[peer];
                    let new = prev.remove(v);
                    if new.is_empty() {
                        return false;
                    }
                    undo.push(UndoEntry {
                        idx: peer,
                        prev_candidates: prev,
                        prev_grid: self.grid[peer],
                    });
                    self.candidates[peer] = new;
                    if let Some(sv) = new.singleton() {
                        propagation_queue.push((peer, sv));
                    }
                }
            }

            // Same column, different row
            if j != r {
                let peer = j * self.n + c;
                if self.grid[peer].is_none() && self.candidates[peer].contains(v) {
                    let prev = self.candidates[peer];
                    let new = prev.remove(v);
                    if new.is_empty() {
                        return false;
                    }
                    undo.push(UndoEntry {
                        idx: peer,
                        prev_candidates: prev,
                        prev_grid: self.grid[peer],
                    });
                    self.candidates[peer] = new;
                    if let Some(sv) = new.singleton() {
                        propagation_queue.push((peer, sv));
                    }
                }
            }
        }

        // Propagate naked singles
        for (pidx, pv) in propagation_queue {
            if self.grid[pidx].is_none() && !self.assign(pidx, pv, undo) {
                return false;
            }
        }

        // Check clue constraints for completed lines
        if !self.check_completed_line_clues(r, c) {
            return false;
        }

        true
    }

    /// Check clue constraints for the row and column containing (r, c),
    /// but only if the entire line is fully assigned.
    fn check_completed_line_clues(&self, r: usize, c: usize) -> bool {
        // Check row only if there is at least one row clue
        if self.left[r].is_some() || self.right[r].is_some() {
            let row_complete = (0..self.n).all(|j| self.grid[r * self.n + j].is_some());
            if row_complete {
                if let Some(expected) = self.left[r] {
                    if self.count_visible_row(r, true) != expected {
                        return false;
                    }
                }
                if let Some(expected) = self.right[r] {
                    if self.count_visible_row(r, false) != expected {
                        return false;
                    }
                }
            }
        }

        // Check column only if there is at least one column clue
        if self.top[c].is_some() || self.bottom[c].is_some() {
            let col_complete = (0..self.n).all(|j| self.grid[j * self.n + c].is_some());
            if col_complete {
                if let Some(expected) = self.top[c] {
                    if self.count_visible_col(c, true) != expected {
                        return false;
                    }
                }
                if let Some(expected) = self.bottom[c] {
                    if self.count_visible_col(c, false) != expected {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Undo assignments recorded in the undo log.
    fn undo(&mut self, undo_log: &[UndoEntry]) {
        for entry in undo_log.iter().rev() {
            self.grid[entry.idx] = entry.prev_grid;
            self.candidates[entry.idx] = entry.prev_candidates;
        }
    }

    /// Recursive search. Finds solutions and appends to `solutions`.
    fn search(&mut self, solutions: &mut Vec<Vec<Option<u8>>>, limit: usize) {
        if solutions.len() >= limit {
            return;
        }

        // Find unassigned cell with minimum remaining values (MRV)
        let mut best: Option<(usize, u32)> = None;
        for (idx, &cand) in self.candidates.iter().enumerate() {
            if self.grid[idx].is_none() {
                let cnt = cand.count();
                if cnt == 0 {
                    return; // contradiction
                }
                if best.is_none() || cnt < best.unwrap().1 {
                    best = Some((idx, cnt));
                }
            }
        }

        let Some((idx, _)) = best else {
            // All cells assigned — verify clue constraints
            if self.verify_clues() {
                solutions.push(self.grid.clone());
            }
            return;
        };

        let cands = self.candidates[idx];
        for v in cands.iter() {
            let mut undo_log = Vec::new();
            if self.assign(idx, v, &mut undo_log) {
                self.search(solutions, limit);
                if solutions.len() >= limit {
                    self.undo(&undo_log);
                    return;
                }
            }
            self.undo(&undo_log);
        }
    }

    /// Verify all clue constraints against the completed grid.
    fn verify_clues(&self) -> bool {
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
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle, Solution};

    use super::*;

    fn make_4x4_solution() -> Solution {
        Solution::new(
            4,
            vec![
                2, 1, 4, 3, //
                3, 4, 1, 2, //
                4, 3, 2, 1, //
                1, 2, 3, 4, //
            ],
        )
    }

    fn make_4x4_puzzle_full() -> Puzzle {
        let sol = make_4x4_solution();
        let clues = Clues::from_solution(&sol);
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        Puzzle { board, clues }
    }

    #[test]
    fn solve_full_board_full_clues() {
        let puzzle = make_4x4_puzzle_full();
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], make_4x4_solution());
    }

    #[test]
    fn solve_empty_board_full_clues() {
        let sol = make_4x4_solution();
        let clues = Clues::from_solution(&sol);
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], sol);
    }

    #[test]
    fn solve_partial_clues_multiple_solutions() {
        let sol = make_4x4_solution();
        let mut clues = Clues::from_solution(&sol);
        // Remove most clues — should allow multiple solutions
        for i in 0..4 {
            clues.set_top(i, None);
            clues.set_bottom(i, None);
            clues.set_left(i, None);
            clues.set_right(i, None);
        }
        // Keep just one clue
        clues.set_top(0, Some(3));
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 10);
        assert!(solutions.len() > 1, "should find multiple solutions");
        // All solutions should satisfy the clue
        for s in &solutions {
            let derived = Clues::from_solution(s);
            assert_eq!(derived.top(0), Some(3));
        }
    }

    #[test]
    fn solve_limit_parameter() {
        let sol = make_4x4_solution();
        let mut clues = Clues::from_solution(&sol);
        for i in 0..4 {
            clues.set_top(i, None);
            clues.set_bottom(i, None);
            clues.set_left(i, None);
            clues.set_right(i, None);
        }
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        // No clues at all — many solutions, but limit to 1
        let solutions = solver.solve(&puzzle, 1);
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn solve_contradictory_input() {
        let mut board = Board::new_empty(4);
        // Place two 1s in the same row
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(1));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 10);
        assert!(solutions.is_empty());
    }

    #[test]
    fn solve_partial_board_no_clues() {
        let sol = make_4x4_solution();
        let mut board = Board::new_empty(4);
        // Place first two rows
        for r in 0..2 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 10);
        // Should find at least one solution that extends the partial board
        assert!(!solutions.is_empty());
        for s in &solutions {
            for r in 0..2 {
                for c in 0..4 {
                    assert_eq!(s.get(r, c), sol.get(r, c));
                }
            }
        }
    }

    #[test]
    fn solve_n7_roundtrip() {
        let cells = vec![
            1, 2, 3, 4, 5, 6, 7, //
            2, 3, 4, 5, 6, 7, 1, //
            3, 4, 5, 6, 7, 1, 2, //
            4, 5, 6, 7, 1, 2, 3, //
            5, 6, 7, 1, 2, 3, 4, //
            6, 7, 1, 2, 3, 4, 5, //
            7, 1, 2, 3, 4, 5, 6, //
        ];
        let sol = Solution::new(7, cells);
        let clues = Clues::from_solution(&sol);
        let board = Board::new_empty(7);
        let puzzle = Puzzle { board, clues };
        let solver = BacktrackingSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], sol);
    }
}
