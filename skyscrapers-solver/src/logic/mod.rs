pub mod difficulty;
mod state;
mod techniques;

use skyscrapers_core::{Puzzle, Solution};

use crate::Solver;
use difficulty::{Difficulty, Step, Technique};
use state::SolveState;
use techniques::{TechniqueResult, apply_next_technique};

/// Result of solving with the logic solver.
pub struct SolveResult {
    pub solutions: Vec<Solution>,
    pub difficulty: Option<Difficulty>,
    pub steps: Vec<Step>,
}

/// A solver that uses human-like logical techniques.
///
/// Applies techniques in order of difficulty. Does not use backtracking;
/// if no technique can make progress, the puzzle is reported as unsolvable.
pub struct LogicSolver;

impl LogicSolver {
    /// Solve the puzzle and return solutions, difficulty, and steps.
    pub fn solve_with_difficulty(&self, puzzle: &Puzzle, limit: usize) -> SolveResult {
        if limit == 0 {
            return SolveResult {
                solutions: Vec::new(),
                difficulty: None,
                steps: Vec::new(),
            };
        }

        let mut state = match SolveState::new(puzzle) {
            Some(s) => s,
            None => {
                return SolveResult {
                    solutions: Vec::new(),
                    difficulty: None,
                    steps: Vec::new(),
                };
            }
        };

        let mut steps = Vec::new();
        let mut max_technique: Option<Technique> = None;

        loop {
            // Check if already complete (initial board values + propagation may solve it)
            if state.is_complete() {
                if state.verify_clues() {
                    return SolveResult {
                        solutions: vec![state.to_solution()],
                        difficulty: Some(
                            max_technique.map_or(Difficulty::Easy, |t| t.difficulty()),
                        ),
                        steps,
                    };
                } else {
                    // Grid is complete but violates clues — contradiction
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
            }

            match apply_next_technique(&mut state) {
                TechniqueResult::Progress(step) => {
                    let technique = step.technique;
                    steps.push(step);
                    match max_technique {
                        Some(current) if technique > current => {
                            max_technique = Some(technique);
                        }
                        None => {
                            max_technique = Some(technique);
                        }
                        _ => {}
                    }
                }
                TechniqueResult::Contradiction => {
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
                TechniqueResult::NoProgress => {
                    // Stuck — cannot solve with logic alone
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
            }
        }
    }

    /// Get the next logical step from the current board state (for hints).
    pub fn next_step(&self, puzzle: &Puzzle, board: &skyscrapers_core::Board) -> Option<Step> {
        // Build a puzzle from the original clues + current board state
        let hint_puzzle = Puzzle {
            board: board.clone(),
            clues: puzzle.clues.clone(),
        };

        let mut state = SolveState::new(&hint_puzzle)?;

        match apply_next_technique(&mut state) {
            TechniqueResult::Progress(step) => Some(step),
            _ => None,
        }
    }
}

impl Solver for LogicSolver {
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution> {
        self.solve_with_difficulty(puzzle, limit).solutions
    }
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle, Solution};

    use super::*;

    #[test]
    fn solve_full_board() {
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let clues = Clues::from_solution(&sol);
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
    }

    #[test]
    fn solve_empty_returns_empty_when_stuck() {
        // Empty board with no clues — logic solver cannot solve
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert!(result.solutions.is_empty());
    }

    #[test]
    fn solve_near_complete_with_naked_singles() {
        // Board missing only a few cells, solvable by naked singles alone
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        // Fill all but last column
        for r in 0..4 {
            for c in 0..3 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
        assert_eq!(result.difficulty, Some(Difficulty::Easy));
    }

    #[test]
    fn solve_with_hidden_singles() {
        // Place values so that hidden singles are needed:
        // col 0 is fully determined, cols 1-3 have gaps requiring hidden singles
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        // Fill column 0 and some diagonal cells to create hidden single opportunities
        for r in 0..4 {
            board.set(r, 0, Some(sol.get(r, 0)));
        }
        // Add enough to force unique solution via hidden singles
        board.set(0, 1, Some(1));
        board.set(1, 1, Some(4));
        board.set(2, 2, Some(2));
        board.set(3, 3, Some(4));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
    }

    #[test]
    fn solve_with_difficulty_reports_easy() {
        // A near-complete board solved entirely during init (propagation)
        // should report Easy difficulty
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..3 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        // Solved during init (no explicit steps), but still Easy
        assert_eq!(result.difficulty, Some(Difficulty::Easy));
    }

    /// Helper to build a puzzle from clue specification.
    fn build_puzzle_with_clues(
        n: usize,
        givens: &[(usize, usize, u8)],
        top: &[(usize, u8)],
        bottom: &[(usize, u8)],
        left: &[(usize, u8)],
        right: &[(usize, u8)],
    ) -> Puzzle {
        let mut board = Board::new_empty(n);
        for &(r, c, v) in givens {
            board.set(r, c, Some(v));
        }
        let mut clues = Clues::new_all_none(n);
        for &(i, v) in top {
            clues.set_top(i, Some(v));
        }
        for &(i, v) in bottom {
            clues.set_bottom(i, Some(v));
        }
        for &(i, v) in left {
            clues.set_left(i, Some(v));
        }
        for &(i, v) in right {
            clues.set_right(i, Some(v));
        }
        Puzzle { board, clues }
    }

    #[test]
    fn solve_n4_seed9_with_forcing_chain() {
        // n=4, seed=9: 0 givens, clues: top=[2,_,2,2], left=[_,_,2,_], bottom=[_,_,2,_]
        // Requires FullForcingChain (Grandmaster) because SimpleForcingChain's basic propagation
        // (NakedSingles + HiddenSingles only) is insufficient to detect contradictions.
        let puzzle =
            build_puzzle_with_clues(4, &[], &[(0, 2), (2, 2), (3, 2)], &[(2, 2)], &[(2, 2)], &[]);
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=9 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Grandmaster));
    }

    #[test]
    fn solve_n4_seed13() {
        // n=4, seed=13: 2 givens, clues: top=[_,2,_,_], left=[_,_,1,_], bottom=[_,_,_,2]
        // givens: (2,1)=2, (3,0)=1
        let puzzle = build_puzzle_with_clues(
            4,
            &[(2, 1, 2), (3, 0, 1)],
            &[(1, 2)],
            &[(3, 2)],
            &[(2, 1)],
            &[],
        );
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=13 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Expert));
    }

    #[test]
    fn solve_n4_seed15() {
        // n=4, seed=15: 0 givens, clues: left=[_,2,_,3], right=[_,_,3,1], bottom=[3,_,_,_]
        let puzzle =
            build_puzzle_with_clues(4, &[], &[], &[(0, 3)], &[(1, 2), (3, 3)], &[(2, 3), (3, 1)]);
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=15 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Expert));
    }

    #[test]
    fn next_step_returns_hint() {
        // Board with a hidden single available
        let mut board = Board::new_empty(4);
        // Place values so that 4 in row 0 can only go at col 2 (hidden single)
        board.set(1, 0, Some(3));
        board.set(1, 1, Some(4));
        board.set(2, 0, Some(4));
        board.set(3, 0, Some(1));
        board.set(3, 3, Some(4));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle {
            board: board.clone(),
            clues,
        };

        let step = LogicSolver.next_step(&puzzle, &board);
        assert!(step.is_some());
        let step = step.unwrap();
        assert!(!step.actions.is_empty());
    }

}
