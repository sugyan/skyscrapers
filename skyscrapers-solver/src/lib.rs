mod backtracking;
mod candidates;
mod sat;

pub use backtracking::BacktrackingSolver;
pub use sat::SatSolver;
use skyscrapers_core::{Puzzle, Solution};

/// A solver that finds solutions to a Skyscrapers puzzle.
pub trait Solver {
    /// Returns up to `limit` solutions for the given puzzle.
    ///
    /// Stops searching as soon as `limit` solutions have been found.
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution>;
}
