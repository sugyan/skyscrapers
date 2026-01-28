//! Jacobson-Matthews algorithm for ergodic Latin square sampling.
//!
//! This algorithm uses a 3D {-1, 0, 1} array representation and can pass through
//! "improper" Latin square states where one position has value -1.
//!
//! Reference: Jacobson, M. T., & Matthews, P. (1996). "Generating uniformly
//! distributed random Latin squares." Journal of Combinatorial Designs, 4(6), 405-437.

use crate::LatinSquare;
use rand::Rng;

/// Internal state for Jacobson-Matthews sampling.
///
/// Uses a 3D array representation: sigma[r][c][s] ∈ {-1, 0, 1}
/// where sigma[r][c][s] = 1 means cell (r,c) contains symbol s.
///
/// In a proper Latin square, all entries are 0 or 1.
/// In an improper state, exactly one entry is -1.
pub(crate) struct JMState {
    n: usize,
    /// Flattened 3D array: sigma[r * n * n + c * n + s]
    sigma: Vec<i8>,
    /// Tracks the position of the -1 entry (if any) for O(1) lookup.
    improper_pos: Option<(usize, usize, usize)>,
}

impl JMState {
    /// Create a new JM state from a cyclic Latin square.
    pub fn new_cyclic(n: usize) -> Self {
        let mut sigma = vec![0i8; n * n * n];
        for r in 0..n {
            for c in 0..n {
                let s = (r + c) % n;
                sigma[r * n * n + c * n + s] = 1;
            }
        }
        Self {
            n,
            sigma,
            improper_pos: None,
        }
    }

    /// Check if the state is proper (no -1 entries). O(1).
    #[inline]
    pub fn is_proper(&self) -> bool {
        self.improper_pos.is_none()
    }

    /// Convert to a LatinSquare. Only valid if proper.
    ///
    /// # Panics
    /// Panics if the state is improper.
    pub fn to_latin_square(&self) -> LatinSquare {
        debug_assert!(self.is_proper(), "cannot convert improper state");
        let n = self.n;
        let mut sq = LatinSquare::new_cyclic(n);
        for r in 0..n {
            for c in 0..n {
                for s in 0..n {
                    if self.get(r, c, s) == 1 {
                        sq.set_unchecked(r, c, s as u8);
                        break;
                    }
                }
            }
        }
        sq
    }

    /// Perform one Jacobson-Matthews move.
    ///
    /// From a proper state, this may transition to an improper state.
    /// From an improper state, this may return to proper or stay improper.
    pub fn step<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let n = self.n;

        if self.is_proper() {
            // Proper state: choose random (r, c, s) where sigma[r][c][s] = 0
            let r = rng.random_range(0..n);
            let c = rng.random_range(0..n);

            // Find current symbol at (r, c)
            let s_orig = self.symbol_at(r, c);

            // Choose a different symbol
            let s = loop {
                let s = rng.random_range(0..n);
                if s != s_orig {
                    break s;
                }
            };

            // In proper state, there's exactly one column with s in row r,
            // exactly one row with s in column c, and exactly one symbol at (r', c').
            let c_prime = self.find_col_with_symbol_in_row(r, s);
            let r_prime = self.find_row_with_symbol_in_col(c, s);
            let s_prime = self.symbol_at(r_prime, c_prime);

            self.apply_move(r, c, s, r_prime, c_prime, s_prime);
        } else {
            // Improper state: find the -1 position
            let (r, c, s) = self.find_minus_one().unwrap();

            // In improper state, there are exactly 2 columns with s in row r,
            // 2 rows with s in column c, and 1-2 symbols at the intersection.
            // Use stack-allocated arrays instead of Vec to avoid allocation.
            let mut cols = [0usize; 2];
            let mut col_count = 0;
            for c2 in 0..n {
                if self.get(r, c2, s) == 1 {
                    cols[col_count] = c2;
                    col_count += 1;
                    if col_count == 2 {
                        break;
                    }
                }
            }

            let mut rows = [0usize; 2];
            let mut row_count = 0;
            for r2 in 0..n {
                if self.get(r2, c, s) == 1 {
                    rows[row_count] = r2;
                    row_count += 1;
                    if row_count == 2 {
                        break;
                    }
                }
            }

            let c_prime = cols[rng.random_range(0..col_count)];
            let r_prime = rows[rng.random_range(0..row_count)];

            // At (r', c'), there may be 1 or 2 symbols with value 1
            let mut symbols = [0usize; 2];
            let mut symbol_count = 0;
            for s2 in 0..n {
                if self.get(r_prime, c_prime, s2) == 1 {
                    symbols[symbol_count] = s2;
                    symbol_count += 1;
                    if symbol_count == 2 {
                        break;
                    }
                }
            }
            let s_prime = symbols[rng.random_range(0..symbol_count)];

            self.apply_move(r, c, s, r_prime, c_prime, s_prime);
        }
    }

    /// Apply the ±1 move on the 2x2x2 cube of positions.
    fn apply_move(
        &mut self,
        r: usize,
        c: usize,
        s: usize,
        r_prime: usize,
        c_prime: usize,
        s_prime: usize,
    ) {
        // Increment: (r,c,s), (r',c',s), (r,c',s'), (r',c,s')
        self.add(r, c, s, 1);
        self.add(r_prime, c_prime, s, 1);
        self.add(r, c_prime, s_prime, 1);
        self.add(r_prime, c, s_prime, 1);

        // Decrement: (r,c',s), (r',c,s), (r,c,s'), (r',c',s')
        self.add(r, c_prime, s, -1);
        self.add(r_prime, c, s, -1);
        self.add(r, c, s_prime, -1);
        self.add(r_prime, c_prime, s_prime, -1);

        // Update improper_pos: exactly one of the decremented positions may become -1.
        // We check all four to find which one (if any) became -1.
        self.improper_pos = None;
        for &(ri, ci, si) in &[
            (r, c_prime, s),
            (r_prime, c, s),
            (r, c, s_prime),
            (r_prime, c_prime, s_prime),
        ] {
            if self.get(ri, ci, si) == -1 {
                self.improper_pos = Some((ri, ci, si));
                break;
            }
        }
    }

    #[inline]
    fn get(&self, r: usize, c: usize, s: usize) -> i8 {
        self.sigma[r * self.n * self.n + c * self.n + s]
    }

    #[inline]
    fn add(&mut self, r: usize, c: usize, s: usize, delta: i8) {
        self.sigma[r * self.n * self.n + c * self.n + s] += delta;
    }

    /// Find the symbol at cell (r, c). Assumes proper state or unique symbol.
    fn symbol_at(&self, r: usize, c: usize) -> usize {
        (0..self.n).find(|&s| self.get(r, c, s) == 1).unwrap()
    }

    /// Find the column in row r that contains symbol s.
    fn find_col_with_symbol_in_row(&self, r: usize, s: usize) -> usize {
        (0..self.n).find(|&c| self.get(r, c, s) == 1).unwrap()
    }

    /// Find the row in column c that contains symbol s.
    fn find_row_with_symbol_in_col(&self, c: usize, s: usize) -> usize {
        (0..self.n).find(|&r| self.get(r, c, s) == 1).unwrap()
    }

    /// Find the position of the -1 entry. O(1).
    #[inline]
    fn find_minus_one(&self) -> Option<(usize, usize, usize)> {
        self.improper_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn new_cyclic_is_proper() {
        for n in 2..=10 {
            let state = JMState::new_cyclic(n);
            assert!(
                state.is_proper(),
                "cyclic state should be proper for n={}",
                n
            );
        }
    }

    #[test]
    fn cyclic_converts_correctly() {
        for n in 2..=10 {
            let state = JMState::new_cyclic(n);
            let sq = state.to_latin_square();
            assert!(
                sq.is_latin(),
                "converted square should be Latin for n={}",
                n
            );
            // Verify it matches the expected cyclic pattern
            for r in 0..n {
                for c in 0..n {
                    assert_eq!(
                        sq.get(r, c) as usize,
                        (r + c) % n,
                        "cell ({},{}) should be {} for n={}",
                        r,
                        c,
                        (r + c) % n,
                        n
                    );
                }
            }
        }
    }

    #[test]
    fn step_preserves_or_returns_to_proper() {
        let mut rng = ChaCha20Rng::from_seed([42u8; 32]);

        for n in 3..=8 {
            let mut state = JMState::new_cyclic(n);

            // Apply many steps
            for _ in 0..10_000 {
                state.step(&mut rng);
            }

            // Eventually we should be in a proper state (with high probability)
            // Run more steps until proper
            for _ in 0..1000 {
                if state.is_proper() {
                    break;
                }
                state.step(&mut rng);
            }

            assert!(
                state.is_proper(),
                "should return to proper state for n={}",
                n
            );

            let sq = state.to_latin_square();
            assert!(sq.is_latin(), "result should be Latin for n={}", n);
        }
    }

    #[test]
    fn move_preserves_latin_after_many_steps() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for n in [7, 8] {
            let mut state = JMState::new_cyclic(n);

            // Apply 50,000 steps
            for _ in 0..50_000 {
                state.step(&mut rng);
            }

            // Run until proper
            while !state.is_proper() {
                state.step(&mut rng);
            }

            let sq = state.to_latin_square();
            assert!(
                sq.is_latin(),
                "after 50k steps, result should be Latin for n={}",
                n
            );
        }
    }

    /// Convert a Latin square to its reduced form.
    /// A reduced form has first row = (0, 1, 2, ..., n-1) and first column = (0, 1, 2, ..., n-1).
    fn to_reduced_form(sq: &LatinSquare) -> Vec<u8> {
        let n = sq.n();

        // Find the permutation that maps first row to (0, 1, ..., n-1)
        // symbol_perm[old_symbol] = new_symbol
        let mut symbol_perm = vec![0u8; n];
        for c in 0..n {
            symbol_perm[sq.get(0, c) as usize] = c as u8;
        }

        // Apply symbol permutation to all cells
        let mut permuted: Vec<Vec<u8>> = vec![vec![0; n]; n];
        for r in 0..n {
            for c in 0..n {
                permuted[r][c] = symbol_perm[sq.get(r, c) as usize];
            }
        }

        // For reduced form:
        // - First row is already (0, 1, ..., n-1) after symbol permutation
        // - For first column to be (0, 1, ..., n-1), we permute rows
        // row_perm[new_row] = old_row (the row that has value new_row in column 0)
        let mut row_perm = vec![0usize; n];
        for r in 0..n {
            row_perm[permuted[r][0] as usize] = r;
        }

        // Apply row permutation
        let mut reduced = Vec::with_capacity(n * n);
        for new_r in 0..n {
            let old_r = row_perm[new_r];
            for c in 0..n {
                reduced.push(permuted[old_r][c]);
            }
        }

        reduced
    }

    #[test]
    fn ergodicity_reaches_multiple_reduced_forms() {
        use std::collections::HashSet;

        // For prime n (like 5 and 7), the old cycle moves would be trapped.
        // Jacobson-Matthews should reach many different reduced forms.
        for n in [5, 7] {
            let mut reduced_forms: HashSet<Vec<u8>> = HashSet::new();
            let num_samples = 200;

            for seed_idx in 0..num_samples {
                let mut seed = [0u8; 32];
                seed[0] = (seed_idx & 0xff) as u8;
                seed[1] = ((seed_idx >> 8) & 0xff) as u8;

                let mut rng = ChaCha20Rng::from_seed(seed);
                let mut state = JMState::new_cyclic(n);

                // Burn-in
                for _ in 0..10_000 {
                    state.step(&mut rng);
                }

                // Ensure proper state
                while !state.is_proper() {
                    state.step(&mut rng);
                }

                let sq = state.to_latin_square();
                let reduced = to_reduced_form(&sq);
                reduced_forms.insert(reduced);
            }

            // For n=5, there are 56 reduced forms. We should reach multiple.
            // For n=7, there are 16,942,080 reduced forms. We should reach many.
            let min_expected = if n == 5 { 10 } else { 50 };
            assert!(
                reduced_forms.len() >= min_expected,
                "n={}: expected at least {} reduced forms, got {}",
                n,
                min_expected,
                reduced_forms.len()
            );

            println!(
                "n={}: reached {} distinct reduced forms from {} samples",
                n,
                reduced_forms.len(),
                num_samples
            );
        }
    }
}
