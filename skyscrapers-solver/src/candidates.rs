/// Bitset representing candidate values for a cell.
///
/// Bit `i` represents value `i + 1`. Supports n up to 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Candidates(u16);

impl Candidates {
    /// Creates a candidate set containing all values 1..=n.
    pub(crate) fn all(n: u8) -> Self {
        debug_assert!(n <= 16);
        Self(((1u32 << n) - 1) as u16)
    }

    /// Creates an empty candidate set.
    pub(crate) fn empty() -> Self {
        Self(0)
    }

    /// Creates a candidate set containing a single value.
    pub(crate) fn single(v: u8) -> Self {
        debug_assert!(v >= 1);
        Self(1u16 << (v - 1))
    }

    /// Returns true if the set contains value `v` (1-based).
    pub(crate) fn contains(self, v: u8) -> bool {
        debug_assert!(v >= 1);
        self.0 & (1u16 << (v - 1)) != 0
    }

    /// Removes value `v` (1-based) from the set. Returns the new set.
    pub(crate) fn remove(self, v: u8) -> Self {
        debug_assert!(v >= 1);
        Self(self.0 & !(1u16 << (v - 1)))
    }

    /// Returns Some(v) if the set contains exactly one value.
    pub(crate) fn singleton(self) -> Option<u8> {
        if self.0 != 0 && (self.0 & (self.0 - 1)) == 0 {
            Some(self.0.trailing_zeros() as u8 + 1)
        } else {
            None
        }
    }

    /// Returns the number of candidates.
    pub(crate) fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if the set is empty.
    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Iterates over all values in the set (1-based).
    pub(crate) fn iter(self) -> CandidatesIter {
        CandidatesIter(self.0)
    }

    /// Returns the intersection of two candidate sets.
    pub(crate) fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns the union of two candidate sets.
    pub(crate) fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

pub(crate) struct CandidatesIter(u16);

impl Iterator for CandidatesIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            let v = self.0.trailing_zeros() as u8 + 1;
            self.0 &= self.0 - 1; // clear lowest set bit
            Some(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_and_contains() {
        let c = Candidates::all(4);
        assert!(c.contains(1));
        assert!(c.contains(2));
        assert!(c.contains(3));
        assert!(c.contains(4));
        assert_eq!(c.count(), 4);
    }

    #[test]
    fn remove_and_singleton() {
        let c = Candidates::all(3);
        let c = c.remove(1).remove(3);
        assert_eq!(c.singleton(), Some(2));
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn empty_set() {
        let c = Candidates::empty();
        assert!(c.is_empty());
        assert_eq!(c.count(), 0);
        assert_eq!(c.singleton(), None);
    }

    #[test]
    fn iter_values() {
        let c = Candidates::all(4).remove(2);
        let vals: Vec<u8> = c.iter().collect();
        assert_eq!(vals, vec![1, 3, 4]);
    }

    #[test]
    fn single_value() {
        let c = Candidates::single(3);
        assert!(c.contains(3));
        assert!(!c.contains(1));
        assert_eq!(c.singleton(), Some(3));
    }

    #[test]
    fn intersect() {
        let a = Candidates::all(4).remove(1);
        let b = Candidates::all(4).remove(4);
        let c = a.intersect(b);
        let vals: Vec<u8> = c.iter().collect();
        assert_eq!(vals, vec![2, 3]);
    }
}
