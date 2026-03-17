use std::fmt;
use std::str::FromStr;

use crate::{Board, Clues, Puzzle};

/// Error type for parsing puzzle text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Not enough lines in the input.
    NotEnoughLines,
    /// A separator line was expected but not found.
    ExpectedSeparator,
    /// A grid row is malformed (missing `|` delimiters or wrong cell count).
    InvalidGridRow(String),
    /// A clue or cell token is invalid.
    InvalidToken(String),
    /// The number of clues does not match the grid size.
    InconsistentSize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NotEnoughLines => write!(f, "not enough lines in puzzle input"),
            ParseError::ExpectedSeparator => write!(f, "expected separator line (+-...+)"),
            ParseError::InvalidGridRow(line) => write!(f, "invalid grid row: {line}"),
            ParseError::InvalidToken(tok) => write!(f, "invalid token: {tok}"),
            ParseError::InconsistentSize => write!(f, "inconsistent size in puzzle input"),
        }
    }
}

impl std::error::Error for ParseError {}

fn parse_token(tok: &str) -> Result<Option<u8>, ParseError> {
    if tok == "." {
        Ok(None)
    } else {
        let v = tok
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidToken(tok.to_string()))?;
        if v == 0 {
            return Err(ParseError::InvalidToken(tok.to_string()));
        }
        Ok(Some(v))
    }
}

fn validate_value(v: Option<u8>, n: usize) -> Result<(), ParseError> {
    if let Some(v) = v {
        if v as usize > n {
            return Err(ParseError::InvalidToken(v.to_string()));
        }
    }
    Ok(())
}

fn is_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3
        && trimmed.starts_with('+')
        && trimmed.ends_with('+')
        && trimmed[1..trimmed.len() - 1].chars().all(|c| c == '-')
}

fn parse_clue_line(line: &str) -> Result<Vec<Option<u8>>, ParseError> {
    line.split_whitespace().map(parse_token).collect()
}

impl FromStr for Puzzle {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();

        if lines.len() < 4 {
            return Err(ParseError::NotEnoughLines);
        }

        // Find separator lines
        let mut sep_indices: Vec<usize> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if is_separator(line) {
                sep_indices.push(i);
            }
        }
        if sep_indices.len() < 2 {
            return Err(ParseError::ExpectedSeparator);
        }

        let first_sep = sep_indices[0];
        let last_sep = *sep_indices.last().unwrap();

        // Top clues: last non-separator line before first separator
        let top_line = lines[..first_sep]
            .last()
            .ok_or(ParseError::NotEnoughLines)?;
        let top_clues = parse_clue_line(top_line)?;
        let n = top_clues.len();
        if !(1..=9).contains(&n) {
            return Err(ParseError::InconsistentSize);
        }

        // Bottom clues: first non-separator line after last separator
        let bottom_line = lines.get(last_sep + 1).ok_or(ParseError::NotEnoughLines)?;
        let bottom_clues = parse_clue_line(bottom_line)?;
        if bottom_clues.len() != n {
            return Err(ParseError::InconsistentSize);
        }

        // Grid rows: lines between separators
        let grid_lines = &lines[first_sep + 1..last_sep];
        if grid_lines.len() != n {
            return Err(ParseError::InconsistentSize);
        }

        let mut board = Board::new_empty(n);
        let mut clues = Clues::new_all_none(n);

        for i in 0..n {
            validate_value(top_clues[i], n)?;
            validate_value(bottom_clues[i], n)?;
            clues.set_top(i, top_clues[i]);
            clues.set_bottom(i, bottom_clues[i]);
        }

        for (r, line) in grid_lines.iter().enumerate() {
            // Split by '|': should give [left_part, cells_part, right_part]
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() != 3 {
                return Err(ParseError::InvalidGridRow(line.to_string()));
            }

            let left_clue = parse_token(parts[0].trim())?;
            validate_value(left_clue, n)?;
            clues.set_left(r, left_clue);

            let right_clue = parse_token(parts[2].trim())?;
            validate_value(right_clue, n)?;
            clues.set_right(r, right_clue);

            let mut c = 0;
            for tok in parts[1].split_whitespace() {
                if c >= n {
                    return Err(ParseError::InvalidGridRow(line.to_string()));
                }
                let val = parse_token(tok)?;
                validate_value(val, n)?;
                board.set(r, c, val);
                c += 1;
            }
            if c != n {
                return Err(ParseError::InvalidGridRow(line.to_string()));
            }
        }

        Ok(Puzzle { board, clues })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_puzzle() {
        let input = "\
    . 3 . . .
  +-----------+
. | . 1 . . 2 | .
3 | . . . . . | .
. | . . . . . | .
. | 4 . . . . | .
. | . . . . . | .
  +-----------+
    . . 2 . .";
        let puzzle: Puzzle = input.parse().unwrap();
        assert_eq!(puzzle.board.n(), 5);
        assert_eq!(puzzle.clues.top(1), Some(3));
        assert_eq!(puzzle.clues.top(0), None);
        assert_eq!(puzzle.clues.bottom(2), Some(2));
        assert_eq!(puzzle.clues.left(1), Some(3));
        assert_eq!(puzzle.board.get(0, 1), Some(1));
        assert_eq!(puzzle.board.get(0, 4), Some(2));
        assert_eq!(puzzle.board.get(3, 0), Some(4));
        assert_eq!(puzzle.board.get(1, 1), None);
    }

    #[test]
    fn display_then_parse_roundtrip() {
        let n = 4;
        let mut board = Board::new_empty(n);
        board.set(0, 1, Some(1));
        board.set(2, 3, Some(4));
        let mut clues = Clues::new_all_none(n);
        clues.set_top(0, Some(3));
        clues.set_top(2, Some(1));
        clues.set_bottom(1, Some(2));
        clues.set_left(0, Some(2));
        clues.set_right(3, Some(1));
        let puzzle = Puzzle { board, clues };

        let text = puzzle.to_string();
        let parsed: Puzzle = text.parse().unwrap();
        assert_eq!(puzzle, parsed);
    }

    #[test]
    fn parse_all_none_clues_and_empty_board() {
        let n = 3;
        let board = Board::new_empty(n);
        let clues = Clues::new_all_none(n);
        let puzzle = Puzzle { board, clues };

        let text = puzzle.to_string();
        let parsed: Puzzle = text.parse().unwrap();
        assert_eq!(puzzle, parsed);
    }

    #[test]
    fn parse_error_not_enough_lines() {
        let result = "just one line".parse::<Puzzle>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_zero_value() {
        let input = "\
    . . .
  +-------+
. | 0 . . | .
. | . . . | .
. | . . . | .
  +-------+
    . . .";
        let result = input.parse::<Puzzle>();
        assert!(matches!(result, Err(ParseError::InvalidToken(_))));
    }

    #[test]
    fn parse_rejects_value_exceeding_n() {
        let input = "\
    . . .
  +-------+
. | 4 . . | .
. | . . . | .
. | . . . | .
  +-------+
    . . .";
        let result = input.parse::<Puzzle>();
        assert!(matches!(result, Err(ParseError::InvalidToken(_))));
    }
}
