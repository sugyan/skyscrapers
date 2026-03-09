/// An n×n board with possibly undetermined cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    n: usize,
    cells: Vec<Option<u8>>,
}

impl Board {
    /// Creates a new empty board of order `n`.
    pub fn new_empty(n: usize) -> Self {
        Self {
            n,
            cells: vec![None; n * n],
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
        self.cells[r * self.n + c]
    }

    /// Sets the value at position (r, c).
    ///
    /// # Panics
    /// Panics if `r >= n` or `c >= n`.
    pub fn set(&mut self, r: usize, c: usize, v: Option<u8>) {
        assert!(r < self.n && c < self.n, "index out of bounds");
        self.cells[r * self.n + c] = v;
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
}
