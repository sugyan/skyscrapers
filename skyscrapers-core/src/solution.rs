/// A completely filled n×n board (solution).
///
/// Cell values are 1-based (`1..=n`), stored in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    n: usize,
    cells: Vec<u8>,
}

impl Solution {
    /// Creates a new `Solution` from raw cells.
    ///
    /// `cells` must have exactly `n * n` elements with values in `1..=n`.
    ///
    /// # Panics
    /// Panics if `n` is 0 or exceeds 255, `cells.len() != n * n`, or any value is out of range.
    pub fn new(n: usize, cells: Vec<u8>) -> Self {
        assert!((1..=255).contains(&n), "n must be in range 1..=255");
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
}
