/// An n×n board with possibly undetermined cells.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Board {
    n: usize,
    cells: Vec<Vec<Option<u8>>>,
}

impl Board {
    /// Creates a new empty board of order `n`.
    ///
    /// # Panics
    /// Panics if `n` is not in `1..=9`.
    pub fn new_empty(n: usize) -> Self {
        assert!((1..=9).contains(&n), "n must be in range 1..=9");
        Self {
            n,
            cells: vec![vec![None; n]; n],
        }
    }

    /// Returns the order of the board.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns the value at position (r, c).
    ///
    /// # Panics
    /// Panics if `r >= n` or `c >= n`.
    pub fn get(&self, r: usize, c: usize) -> Option<u8> {
        assert!(r < self.n && c < self.n, "index out of bounds");
        self.cells[r][c]
    }

    /// Sets the value at position (r, c).
    ///
    /// # Panics
    /// Panics if `r >= n`, `c >= n`, or `v` is `Some(x)` with `x` outside `1..=n`.
    pub fn set(&mut self, r: usize, c: usize, v: Option<u8>) {
        assert!(r < self.n && c < self.n, "index out of bounds");
        if let Some(x) = v {
            assert!(x >= 1 && x <= self.n as u8, "cell value must be in 1..=n");
        }
        self.cells[r][c] = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_empty_and_set() {
        let mut board = Board::new_empty(4);
        assert_eq!(board.n(), 4);
        assert_eq!(board.get(0, 0), None);

        board.set(1, 2, Some(3));
        assert_eq!(board.get(1, 2), Some(3));

        board.set(1, 2, None);
        assert_eq!(board.get(1, 2), None);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn board_get_out_of_bounds() {
        let board = Board::new_empty(3);
        board.get(3, 0);
    }

    #[test]
    #[should_panic(expected = "n must be in range 1..=9")]
    fn board_n_too_large() {
        Board::new_empty(10);
    }

    #[test]
    #[should_panic(expected = "cell value must be in 1..=n")]
    fn board_set_value_zero() {
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(0));
    }

    #[test]
    #[should_panic(expected = "cell value must be in 1..=n")]
    fn board_set_value_too_large() {
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(5));
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn board_set_out_of_bounds() {
        let mut board = Board::new_empty(3);
        board.set(3, 0, Some(1));
    }
}
