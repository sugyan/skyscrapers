use rand::Rng;

use crate::LatinSquare;

const MAX_CYCLE_RETRIES: usize = 8;

/// Performs a row-cycle move on the Latin square.
///
/// This move always preserves the Latin property.
pub(crate) fn row_cycle_move<R: Rng + ?Sized>(sq: &mut LatinSquare, rng: &mut R) {
    let n = sq.n();
    if n < 2 {
        return;
    }

    // 1) Choose distinct rows r1 != r2 uniformly
    let r1 = rng.random_range(0..n);
    let r2 = loop {
        let r = rng.random_range(0..n);
        if r != r1 {
            break r;
        }
    };

    // 2) Build inverse map for row r1: pos[s] = c where L[r1][c] == s
    let mut pos = vec![0usize; n];
    for c in 0..n {
        let s = sq.get(r1, c) as usize;
        pos[s] = c;
    }

    // 3) Define permutation: perm[s] = L[r2][pos[s]]
    let mut perm = vec![0u8; n];
    for s in 0..n {
        perm[s] = sq.get(r2, pos[s]);
    }

    // 4) Select a random non-trivial cycle from perm
    let Some(cycle) = random_nontrivial_cycle(&perm, rng) else {
        return; // No-op if no non-trivial cycle found
    };

    // 5) For each symbol s in the cycle, swap L[r1][pos[s]] and L[r2][pos[s]]
    for &s in &cycle {
        let c = pos[s as usize];
        let v1 = sq.get(r1, c);
        let v2 = sq.get(r2, c);
        sq.set_unchecked(r1, c, v2);
        sq.set_unchecked(r2, c, v1);
    }
}

/// Performs a column-cycle move on the Latin square.
///
/// This move always preserves the Latin property.
/// Symmetric to row_cycle_move, swapping columns c1, c2 using inverse mapping on column c1.
pub(crate) fn col_cycle_move<R: Rng + ?Sized>(sq: &mut LatinSquare, rng: &mut R) {
    let n = sq.n();
    if n < 2 {
        return;
    }

    // 1) Choose distinct columns c1 != c2 uniformly
    let c1 = rng.random_range(0..n);
    let c2 = loop {
        let c = rng.random_range(0..n);
        if c != c1 {
            break c;
        }
    };

    // 2) Build inverse map for column c1: pos[s] = r where L[r][c1] == s
    let mut pos = vec![0usize; n];
    for r in 0..n {
        let s = sq.get(r, c1) as usize;
        pos[s] = r;
    }

    // 3) Define permutation: perm[s] = L[pos[s]][c2]
    let mut perm = vec![0u8; n];
    for s in 0..n {
        perm[s] = sq.get(pos[s], c2);
    }

    // 4) Select a random non-trivial cycle from perm
    let Some(cycle) = random_nontrivial_cycle(&perm, rng) else {
        return; // No-op if no non-trivial cycle found
    };

    // 5) For each symbol s in the cycle, swap L[pos[s]][c1] and L[pos[s]][c2]
    for &s in &cycle {
        let r = pos[s as usize];
        let v1 = sq.get(r, c1);
        let v2 = sq.get(r, c2);
        sq.set_unchecked(r, c1, v2);
        sq.set_unchecked(r, c2, v1);
    }
}

/// Finds a random non-trivial cycle (length >= 2) in the permutation.
///
/// Returns `None` if only fixed points are found after retries.
pub(crate) fn random_nontrivial_cycle<R: Rng + ?Sized>(
    perm: &[u8],
    rng: &mut R,
) -> Option<Vec<u8>> {
    let n = perm.len();
    if n < 2 {
        return None;
    }

    for _ in 0..MAX_CYCLE_RETRIES {
        // Choose random start
        let s0 = rng.random_range(0..n) as u8;

        // Follow the permutation to find the cycle
        let mut cycle = vec![s0];
        let mut current = perm[s0 as usize];
        while current != s0 {
            cycle.push(current);
            current = perm[current as usize];
        }

        // If cycle length >= 2, return it
        if cycle.len() >= 2 {
            return Some(cycle);
        }
        // Otherwise it's a fixed point, retry
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn move_preserves_latin() {
        for n in [7, 8] {
            let mut sq = LatinSquare::new_cyclic(n);
            let mut rng = ChaCha20Rng::from_seed([42u8; 32]);

            for i in 0..50_000 {
                if rng.random::<bool>() {
                    row_cycle_move(&mut sq, &mut rng);
                } else {
                    col_cycle_move(&mut sq, &mut rng);
                }
                assert!(
                    sq.is_latin(),
                    "Latin property violated at step {} for n={}",
                    i,
                    n
                );
            }
        }
    }
}
