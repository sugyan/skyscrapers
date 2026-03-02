use crate::Solution;

/// Clue numbers for all four directions.
///
/// Each direction has `n` clue slots, where `None` means the clue is hidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clues {
    n: usize,
    top: Vec<Option<u8>>,
    bottom: Vec<Option<u8>>,
    left: Vec<Option<u8>>,
    right: Vec<Option<u8>>,
}

impl Clues {
    /// Creates clues with all values set to `None`.
    pub fn new_all_none(n: usize) -> Self {
        Self {
            n,
            top: vec![None; n],
            bottom: vec![None; n],
            left: vec![None; n],
            right: vec![None; n],
        }
    }

    /// Derives all clues from a solution.
    pub fn from_solution(solution: &Solution) -> Self {
        let n = solution.n();
        let mut clues = Self::new_all_none(n);

        for i in 0..n {
            // Top clue for column i: looking down from row 0
            let col: Vec<u8> = (0..n).map(|r| solution.get(r, i)).collect();
            clues.top[i] = Some(count_visible(&col));

            // Bottom clue for column i: looking up from row n-1
            let col_rev: Vec<u8> = (0..n).rev().map(|r| solution.get(r, i)).collect();
            clues.bottom[i] = Some(count_visible(&col_rev));

            // Left clue for row i: looking right from column 0
            let row: Vec<u8> = (0..n).map(|c| solution.get(i, c)).collect();
            clues.left[i] = Some(count_visible(&row));

            // Right clue for row i: looking left from column n-1
            let row_rev: Vec<u8> = (0..n).rev().map(|c| solution.get(i, c)).collect();
            clues.right[i] = Some(count_visible(&row_rev));
        }

        clues
    }

    /// Returns the order.
    pub fn n(&self) -> usize {
        self.n
    }

    pub fn top(&self, i: usize) -> Option<u8> {
        self.top[i]
    }

    pub fn bottom(&self, i: usize) -> Option<u8> {
        self.bottom[i]
    }

    pub fn left(&self, i: usize) -> Option<u8> {
        self.left[i]
    }

    pub fn right(&self, i: usize) -> Option<u8> {
        self.right[i]
    }

    pub fn set_top(&mut self, i: usize, v: Option<u8>) {
        self.top[i] = v;
    }

    pub fn set_bottom(&mut self, i: usize, v: Option<u8>) {
        self.bottom[i] = v;
    }

    pub fn set_left(&mut self, i: usize, v: Option<u8>) {
        self.left[i] = v;
    }

    pub fn set_right(&mut self, i: usize, v: Option<u8>) {
        self.right[i] = v;
    }
}

/// Counts the number of visible buildings from the given viewing direction.
///
/// A building of height `h` is visible if no taller building appears before it.
fn count_visible(heights: &[u8]) -> u8 {
    let mut max = 0u8;
    let mut count = 0u8;
    for &h in heights {
        if h > max {
            count += 1;
            max = h;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_visible_basic() {
        assert_eq!(count_visible(&[2, 1, 4, 3]), 2); // 2 visible, then 4
        assert_eq!(count_visible(&[1, 2, 3, 4]), 4); // all visible
        assert_eq!(count_visible(&[4, 3, 2, 1]), 1); // only first
        assert_eq!(count_visible(&[3, 4, 1, 2]), 2); // 3, then 4
    }

    #[test]
    fn clues_from_solution() {
        // 4×4 solution:
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
        let clues = Clues::from_solution(&sol);

        // Top (looking down each column):
        // col 0: [2,3,4,1] → 3 visible (2,3,4)
        // col 1: [1,4,3,2] → 2 visible (1,4)
        // col 2: [4,1,2,3] → 1 visible (4)
        // col 3: [3,2,1,4] → 2 visible (3,4)
        assert_eq!(clues.top(0), Some(3));
        assert_eq!(clues.top(1), Some(2));
        assert_eq!(clues.top(2), Some(1));
        assert_eq!(clues.top(3), Some(2));

        // Bottom (looking up each column):
        // col 0: [1,4,3,2] → 2 visible (1,4)
        // col 1: [2,3,4,1] → 3 visible (2,3,4)
        // col 2: [3,2,1,4] → 2 visible (3,4)
        // col 3: [4,1,2,3] → 1 visible (4)
        assert_eq!(clues.bottom(0), Some(2));
        assert_eq!(clues.bottom(1), Some(3));
        assert_eq!(clues.bottom(2), Some(2));
        assert_eq!(clues.bottom(3), Some(1));

        // Left (looking right each row):
        // row 0: [2,1,4,3] → 2 visible (2,4)
        // row 1: [3,4,1,2] → 2 visible (3,4)
        // row 2: [4,3,2,1] → 1 visible (4)
        // row 3: [1,2,3,4] → 4 visible (1,2,3,4)
        assert_eq!(clues.left(0), Some(2));
        assert_eq!(clues.left(1), Some(2));
        assert_eq!(clues.left(2), Some(1));
        assert_eq!(clues.left(3), Some(4));

        // Right (looking left each row):
        // row 0: [3,4,1,2] → 2 visible (3,4)
        // row 1: [2,1,4,3] → 2 visible (2,4)  wait...
        // Actually right means looking from right side, so reversed:
        // row 0: [3,4,1,2] → 2 visible (3,4)
        // row 1: [2,1,4,3] → 2 visible (2,4)
        // row 2: [1,2,3,4] → 4 visible
        // row 3: [4,3,2,1] → 1 visible
        assert_eq!(clues.right(0), Some(2));
        assert_eq!(clues.right(1), Some(2));
        assert_eq!(clues.right(2), Some(4));
        assert_eq!(clues.right(3), Some(1));
    }

    #[test]
    fn clues_new_all_none() {
        let clues = Clues::new_all_none(5);
        assert_eq!(clues.n(), 5);
        for i in 0..5 {
            assert_eq!(clues.top(i), None);
            assert_eq!(clues.bottom(i), None);
            assert_eq!(clues.left(i), None);
            assert_eq!(clues.right(i), None);
        }
    }

    #[test]
    fn clues_setters() {
        let mut clues = Clues::new_all_none(3);
        clues.set_top(0, Some(2));
        clues.set_bottom(1, Some(3));
        clues.set_left(2, Some(1));
        clues.set_right(0, Some(2));
        assert_eq!(clues.top(0), Some(2));
        assert_eq!(clues.bottom(1), Some(3));
        assert_eq!(clues.left(2), Some(1));
        assert_eq!(clues.right(0), Some(2));
    }
}
