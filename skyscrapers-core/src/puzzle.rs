use crate::{Board, Clues};

/// A puzzle consists of an initial board and clues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub board: Board,
    pub clues: Clues,
}
