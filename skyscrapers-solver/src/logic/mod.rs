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
                return SolveResult {
                    solutions: vec![state.to_solution()],
                    difficulty: Some(max_technique.map_or(Difficulty::Easy, |t| t.difficulty())),
                    steps,
                };
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
}
