/// A Latin square of order `n`.
///
/// A Latin square is an `n x n` array with symbols `{0..n-1}` such that
/// each row and each column is a permutation of `{0..n-1}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatinSquare {
    n: usize,
    cells: Vec<u8>,
}

impl LatinSquare {
    /// Creates the cyclic Latin square of order `n`: `L[r][c] = (r + c) mod n`.
    ///
    /// # Panics
    /// Panics if `n < 2` or `n > 255`.
    pub fn new_cyclic(n: usize) -> Self {
        assert!(n >= 2 && n <= 255, "n must be in range 2..=255");
        let cells = (0..n)
            .flat_map(|r| (0..n).map(move |c| ((r + c) % n) as u8))
            .collect();
        Self { n, cells }
    }

    /// Returns the order of the Latin square.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns the value at position `(r, c)`.
    ///
    /// # Panics
    /// Panics if `r >= n` or `c >= n`.
    pub fn get(&self, r: usize, c: usize) -> u8 {
        assert!(r < self.n && c < self.n, "index out of bounds");
        self.cells[r * self.n + c]
    }

    /// Sets the value at position `(r, c)` without checking the Latin property.
    pub(crate) fn set_unchecked(&mut self, r: usize, c: usize, v: u8) {
        self.cells[r * self.n + c] = v;
    }

    /// Checks if this square satisfies the Latin property.
    ///
    /// Returns `true` if each row and each column is a permutation of `{0..n-1}`.
    /// This is O(n^2) and intended for debugging/validation.
    pub fn is_latin(&self) -> bool {
        let n = self.n;
        let mut seen = vec![false; n];

        // Check rows
        for r in 0..n {
            seen.fill(false);
            for c in 0..n {
                let v = self.cells[r * n + c] as usize;
                if v >= n || seen[v] {
                    return false;
                }
                seen[v] = true;
            }
        }

        // Check columns
        for c in 0..n {
            seen.fill(false);
            for r in 0..n {
                let v = self.cells[r * n + c] as usize;
                if v >= n || seen[v] {
                    return false;
                }
                seen[v] = true;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_is_latin() {
        for n in 2..=10 {
            let sq = LatinSquare::new_cyclic(n);
            assert!(sq.is_latin(), "cyclic square of order {} should be Latin", n);
        }
    }
}
