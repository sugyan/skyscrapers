use std::fmt;

use crate::{Board, Clues};

/// A puzzle consists of an initial board and clues.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Puzzle {
    pub board: Board,
    pub clues: Clues,
}

fn format_clue(v: Option<u8>) -> char {
    match v {
        Some(v) => char::from(b'0' + v),
        None => '.',
    }
}

fn write_separator(f: &mut fmt::Formatter<'_>, n: usize) -> fmt::Result {
    write!(f, "  +")?;
    for _ in 0..n {
        write!(f, "--")?;
    }
    writeln!(f, "-+")
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.board.n();

        // Top clues
        write!(f, "    ")?;
        for i in 0..n {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", format_clue(self.clues.top(i)))?;
        }
        writeln!(f)?;

        write_separator(f, n)?;

        // Grid rows
        for r in 0..n {
            write!(f, "{} |", format_clue(self.clues.left(r)))?;
            for c in 0..n {
                write!(f, " {}", format_clue(self.board.get(r, c)))?;
            }
            writeln!(f, " | {}", format_clue(self.clues.right(r)))?;
        }

        write_separator(f, n)?;

        // Bottom clues
        write!(f, "    ")?;
        for i in 0..n {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", format_clue(self.clues.bottom(i)))?;
        }

        Ok(())
    }
}
