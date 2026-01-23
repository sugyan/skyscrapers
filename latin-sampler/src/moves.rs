use rand::Rng;

use crate::LatinSquare;

/// Performs a row-cycle move on the Latin square.
///
/// This move always preserves the Latin property.
pub(crate) fn row_cycle_move<R: Rng + ?Sized>(sq: &mut LatinSquare, rng: &mut R) {
    todo!()
}

/// Performs a column-cycle move on the Latin square.
///
/// This move always preserves the Latin property.
pub(crate) fn col_cycle_move<R: Rng + ?Sized>(sq: &mut LatinSquare, rng: &mut R) {
    todo!()
}

/// Finds a random non-trivial cycle (length >= 2) in the permutation.
///
/// Returns `None` if only fixed points are found after retries.
pub(crate) fn random_nontrivial_cycle<R: Rng + ?Sized>(
    perm: &[u8],
    rng: &mut R,
) -> Option<Vec<u8>> {
    todo!()
}
