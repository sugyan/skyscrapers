mod backtracking;
mod candidates;
pub mod logic;

pub use backtracking::BacktrackingSolver;
pub use logic::LogicSolver;
pub use logic::difficulty::Difficulty;
use skyscrapers_core::{Puzzle, Solution};

/// A solver that finds solutions to a Skyscrapers puzzle.
pub trait Solver {
    /// Returns up to `limit` solutions for the given puzzle.
    ///
    /// Stops searching as soon as `limit` solutions have been found.
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution>;
}
