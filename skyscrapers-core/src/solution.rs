use std::fmt;
use std::str::FromStr;

/// A completely filled n×n board (solution).
///
/// Cell values are 1-based (`1..=n`), stored in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    n: usize,
    cells: Vec<u8>,
}

/// Error type for parsing a `Solution` from text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolutionParseError {
    /// The input has no non-empty rows.
    Empty,
    /// Row lengths are inconsistent (not all rows have the same number of columns).
    InconsistentRowLength,
    /// A token is not a valid digit.
    InvalidToken(String),
    /// The grid size is out of the supported range (1..=9).
    InvalidSize(usize),
    /// A cell value is out of range for the grid size.
    ValueOutOfRange(u8, usize),
}

impl fmt::Display for SolutionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::InconsistentRowLength => write!(f, "inconsistent row length"),
            Self::InvalidToken(tok) => write!(f, "invalid token: {tok}"),
            Self::InvalidSize(n) => write!(f, "invalid grid size: {n}"),
            Self::ValueOutOfRange(v, n) => write!(f, "value {v} out of range for n={n}"),
        }
    }
}

impl std::error::Error for SolutionParseError {}

impl Solution {
    /// Creates a new `Solution` from raw cells.
    ///
    /// `cells` must have exactly `n * n` elements with values in `1..=n`.
    ///
    /// # Panics
    /// Panics if `n` is not in `1..=9`, `cells.len() != n * n`, or any value is out of range.
    pub fn new(n: usize, cells: Vec<u8>) -> Self {
        assert!((1..=9).contains(&n), "n must be in range 1..=9");
        assert_eq!(cells.len(), n * n, "cells length must be n*n");
        assert!(
            cells.iter().all(|&v| v >= 1 && v <= n as u8),
            "all cell values must be in 1..=n"
        );
        Self { n, cells }
    }

    /// Returns the order of the solution.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns the value at position (r, c).
    ///
    /// # Panics
    /// Panics if `r >= n` or `c >= n`.
    pub fn get(&self, r: usize, c: usize) -> u8 {
        assert!(r < self.n && c < self.n, "index out of bounds");
        self.cells[r * self.n + c]
    }

    /// Returns the cells as a slice.
    pub fn cells(&self) -> &[u8] {
        &self.cells
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.n {
            if r > 0 {
                writeln!(f)?;
            }
            for c in 0..self.n {
                if c > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", self.cells[r * self.n + c])?;
            }
        }
        Ok(())
    }
}

impl FromStr for Solution {
    type Err = SolutionParseError;

    /// Parses a solution from space-separated rows of digits.
    ///
    /// Example input:
    /// ```text
    /// 1 2 3
    /// 2 3 1
    /// 3 1 2
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows: Vec<Vec<&str>> = s
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .filter(|tokens| !tokens.is_empty())
            .collect();

        if rows.is_empty() {
            return Err(SolutionParseError::Empty);
        }

        let n = rows.len();
        if !rows.iter().all(|row| row.len() == n) {
            return Err(SolutionParseError::InconsistentRowLength);
        }
        if !(1..=9).contains(&n) {
            return Err(SolutionParseError::InvalidSize(n));
        }

        let mut cells = Vec::with_capacity(n * n);
        for row in &rows {
            for &tok in row {
                let v: u8 = tok
                    .parse()
                    .map_err(|_| SolutionParseError::InvalidToken(tok.to_string()))?;
                if v < 1 || v > n as u8 {
                    return Err(SolutionParseError::ValueOutOfRange(v, n));
                }
                cells.push(v);
            }
        }

        Ok(Self { n, cells })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solution_new_and_accessors() {
        // 3×3 solution
        let cells = vec![1, 2, 3, 2, 3, 1, 3, 1, 2];
        let sol = Solution::new(3, cells.clone());
        assert_eq!(sol.n(), 3);
        assert_eq!(sol.get(0, 0), 1);
        assert_eq!(sol.get(1, 2), 1);
        assert_eq!(sol.get(2, 1), 1);
        assert_eq!(sol.cells(), &cells[..]);
    }

    #[test]
    #[should_panic(expected = "cells length must be n*n")]
    fn solution_wrong_length() {
        Solution::new(3, vec![1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "all cell values must be in 1..=n")]
    fn solution_value_out_of_range() {
        Solution::new(3, vec![0, 1, 2, 1, 2, 3, 2, 3, 1]);
    }

    #[test]
    #[should_panic(expected = "n must be in range 1..=9")]
    fn solution_n_zero() {
        Solution::new(0, vec![]);
    }

    #[test]
    #[should_panic(expected = "n must be in range 1..=9")]
    fn solution_n_too_large() {
        Solution::new(10, vec![1; 100]);
    }

    #[test]
    fn display_solution() {
        let sol = Solution::new(3, vec![1, 2, 3, 2, 3, 1, 3, 1, 2]);
        let expected = "1 2 3\n2 3 1\n3 1 2";
        assert_eq!(sol.to_string(), expected);
    }

    #[test]
    fn parse_solution() {
        let input = "1 2 3\n2 3 1\n3 1 2";
        let sol: Solution = input.parse().unwrap();
        assert_eq!(sol.n(), 3);
        assert_eq!(sol.cells(), &[1, 2, 3, 2, 3, 1, 3, 1, 2]);
    }

    #[test]
    fn parse_solution_with_extra_whitespace() {
        let input = "
            1 2 3
            2 3 1
            3 1 2
        ";
        let sol: Solution = input.parse().unwrap();
        assert_eq!(sol.n(), 3);
    }

    #[test]
    fn display_parse_roundtrip() {
        let sol = Solution::new(4, vec![2, 1, 4, 3, 3, 4, 1, 2, 4, 3, 2, 1, 1, 2, 3, 4]);
        let parsed: Solution = sol.to_string().parse().unwrap();
        assert_eq!(sol, parsed);
    }

    #[test]
    fn parse_empty_input() {
        let result: Result<Solution, _> = "".parse();
        assert_eq!(result, Err(SolutionParseError::Empty));
    }

    #[test]
    fn parse_inconsistent_rows() {
        let result: Result<Solution, _> = "1 2 3\n1 2\n3 1 2".parse();
        assert_eq!(result, Err(SolutionParseError::InconsistentRowLength));
    }

    #[test]
    fn parse_value_out_of_range() {
        let result: Result<Solution, _> = "1 2 3\n2 3 1\n3 1 4".parse();
        assert_eq!(result, Err(SolutionParseError::ValueOutOfRange(4, 3)));
    }
}
