use latin_sampler::LatinSquare;
use skyscrapers_core::{Clues, Solution};

/// Converts a `LatinSquare` (0-based symbols) into a `Solution` (1-based heights).
pub fn solution_from_latin_square(ls: &LatinSquare) -> Solution {
    let n = ls.n();
    let cells: Vec<u8> = ls.cells().iter().map(|&v| v + 1).collect();
    Solution::new(n, cells)
}

/// Derives all clues from a solution.
pub fn derive_clues(solution: &Solution) -> Clues {
    Clues::from_solution(solution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use latin_sampler::SamplerParams;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn sample_latin_square(n: usize, seed: u64) -> LatinSquare {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        latin_sampler::sample(n, &mut rng, &SamplerParams::default())
    }

    #[test]
    fn solution_from_latin_square_converts_to_1_based() {
        let ls = sample_latin_square(4, 42);
        let sol = solution_from_latin_square(&ls);

        assert_eq!(sol.n(), 4);
        // Every cell should be 1-based (1..=n)
        for r in 0..4 {
            for c in 0..4 {
                let v = sol.get(r, c);
                assert!(v >= 1 && v <= 4, "cell ({r},{c}) = {v}, expected 1..=4");
                // Should be exactly ls value + 1
                assert_eq!(v, ls.get(r, c) + 1);
            }
        }
    }

    #[test]
    fn solution_is_valid_latin_square() {
        let ls = sample_latin_square(5, 123);
        let sol = solution_from_latin_square(&ls);
        let n = sol.n();

        // Each row is a permutation of 1..=n
        for r in 0..n {
            let mut row: Vec<u8> = (0..n).map(|c| sol.get(r, c)).collect();
            row.sort();
            assert_eq!(row, (1..=n as u8).collect::<Vec<_>>());
        }
        // Each column is a permutation of 1..=n
        for c in 0..n {
            let mut col: Vec<u8> = (0..n).map(|r| sol.get(r, c)).collect();
            col.sort();
            assert_eq!(col, (1..=n as u8).collect::<Vec<_>>());
        }
    }

    #[test]
    fn derive_clues_all_present() {
        let ls = sample_latin_square(4, 42);
        let sol = solution_from_latin_square(&ls);
        let clues = derive_clues(&sol);

        assert_eq!(clues.n(), 4);
        // All clues should be Some and in range 1..=n
        for i in 0..4 {
            for v in [clues.top(i), clues.bottom(i), clues.left(i), clues.right(i)] {
                assert!(v.is_some());
                let v = v.unwrap();
                assert!(v >= 1 && v <= 4, "clue value {v} out of range");
            }
        }
    }

    #[test]
    fn derive_clues_known_row() {
        // Construct a known solution to verify clue derivation
        // 4×4:
        // 2 1 4 3
        // 3 4 1 2
        // 4 3 2 1
        // 1 2 3 4
        let sol = Solution::new(
            4,
            vec![
                2, 1, 4, 3, //
                3, 4, 1, 2, //
                4, 3, 2, 1, //
                1, 2, 3, 4, //
            ],
        );
        let clues = derive_clues(&sol);

        // Left clue for row 3: [1,2,3,4] → 4 visible
        assert_eq!(clues.left(3), Some(4));
        // Right clue for row 2: [1,2,3,4] reversed → 4 visible... no
        // row 2 reversed: [1,2,3,4] → 4 visible
        assert_eq!(clues.right(2), Some(4));
        // Left clue for row 2: [4,3,2,1] → 1 visible
        assert_eq!(clues.left(2), Some(1));
        // Right clue for row 3: [4,3,2,1] → 1 visible
        assert_eq!(clues.right(3), Some(1));
    }
}
