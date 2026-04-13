use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// An Almost Locked Set: k cells with k+1 candidate values in a single line.
struct Als {
    cells: Vec<(usize, usize)>,
    candidates: Candidates,
}

/// ALS-XZ: Two ALSs connected by a restricted common candidate (RCC).
///
/// If ALS A has k cells with k+1 values, and ALS B has m cells with m+1 values,
/// and they share a restricted common candidate x (x appears in both, and all
/// x-cells in A see all x-cells in B), then any other common value z can be
/// eliminated from cells that see all z-cells in A and all z-cells in B.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let all_als = collect_all_als(state);

    for i in 0..all_als.len() {
        for j in (i + 1)..all_als.len() {
            let a = &all_als[i];
            let b = &all_als[j];

            // ALSs must not overlap
            if cells_overlap(&a.cells, &b.cells) {
                continue;
            }

            let common = a.candidates.intersect(b.candidates);
            if common.count() < 2 {
                // Need at least 2 common values: one for RCC (x), one for elimination (z)
                continue;
            }

            // Try each common value as RCC
            for x in common.iter() {
                if !is_restricted_common(state, a, b, x) {
                    continue;
                }

                // Try each other common value as elimination target
                for z in common.iter() {
                    if z == x {
                        continue;
                    }

                    let result = try_eliminate(state, a, b, x, z);
                    if !matches!(result, TechniqueResult::NoProgress) {
                        return result;
                    }
                }
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Collect all ALSs from all rows and columns.
fn collect_all_als(state: &SolveState) -> Vec<Als> {
    let n = state.n;
    let mut result = Vec::new();

    // Rows
    for r in 0..n {
        let unassigned: Vec<(usize, usize)> = (0..n)
            .filter(|&c| state.grid[r * n + c].is_none())
            .map(|c| (r, c))
            .collect();
        collect_als_from_cells(state, &unassigned, &mut result);
    }

    // Columns
    for c in 0..n {
        let unassigned: Vec<(usize, usize)> = (0..n)
            .filter(|&r| state.grid[r * n + c].is_none())
            .map(|r| (r, c))
            .collect();
        collect_als_from_cells(state, &unassigned, &mut result);
    }

    result
}

/// Enumerate subsets of `cells` that form ALSs (k cells, k+1 candidates).
fn collect_als_from_cells(
    state: &SolveState,
    cells: &[(usize, usize)],
    out: &mut Vec<Als>,
) {
    let len = cells.len();
    if len < 2 {
        return;
    }

    // Enumerate subsets of size 2..len-1 using bitmask
    // (size 1 = bivalue cell, handled by XY-Wing; size len = locked set, not almost-locked)
    let max_mask = 1u32 << len;
    for mask in 3..max_mask {
        // mask must have at least 2 bits set
        let size = mask.count_ones() as usize;
        if size < 2 || size >= len {
            continue;
        }

        let mut union_bits = 0u16;
        let mut subset_cells = Vec::with_capacity(size);

        for bit in 0..len {
            if mask & (1 << bit) != 0 {
                let (r, c) = cells[bit];
                let idx = state.idx(r, c);
                union_bits |= state.candidates[idx].raw();
                subset_cells.push((r, c));
            }
        }

        let union = Candidates::from_raw(union_bits);
        let num_values = union.count() as usize;

        if num_values == size + 1 {
            out.push(Als {
                cells: subset_cells,
                candidates: union,
            });
        }
    }
}

/// Check if two cell sets overlap.
fn cells_overlap(a: &[(usize, usize)], b: &[(usize, usize)]) -> bool {
    a.iter().any(|cell| b.contains(cell))
}

/// Check if value x is a restricted common candidate between ALSs A and B.
/// All x-cells in A must see all x-cells in B (share a row or column).
fn is_restricted_common(state: &SolveState, a: &Als, b: &Als, x: u8) -> bool {
    let x_cells_a: Vec<(usize, usize)> = a
        .cells
        .iter()
        .filter(|&&(r, c)| state.candidates[state.idx(r, c)].contains(x))
        .copied()
        .collect();
    let x_cells_b: Vec<(usize, usize)> = b
        .cells
        .iter()
        .filter(|&&(r, c)| state.candidates[state.idx(r, c)].contains(x))
        .copied()
        .collect();

    if x_cells_a.is_empty() || x_cells_b.is_empty() {
        return false;
    }

    // Every cell in x_cells_a must see every cell in x_cells_b
    for &(ar, ac) in &x_cells_a {
        for &(br, bc) in &x_cells_b {
            if ar != br && ac != bc {
                return false;
            }
        }
    }

    true
}

/// Try to eliminate value z from cells that see all z-cells in both ALSs.
fn try_eliminate(
    state: &mut SolveState,
    a: &Als,
    b: &Als,
    rcc_value: u8,
    z: u8,
) -> TechniqueResult {
    let n = state.n;

    let z_cells_a: Vec<(usize, usize)> = a
        .cells
        .iter()
        .filter(|&&(r, c)| state.candidates[state.idx(r, c)].contains(z))
        .copied()
        .collect();
    let z_cells_b: Vec<(usize, usize)> = b
        .cells
        .iter()
        .filter(|&&(r, c)| state.candidates[state.idx(r, c)].contains(z))
        .copied()
        .collect();

    if z_cells_a.is_empty() || z_cells_b.is_empty() {
        return TechniqueResult::NoProgress;
    }

    let mut actions = Vec::new();

    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if state.grid[idx].is_some() || !state.candidates[idx].contains(z) {
                continue;
            }
            // Must not be part of either ALS
            if a.cells.contains(&(r, c)) || b.cells.contains(&(r, c)) {
                continue;
            }
            // Must see all z-cells in A and all z-cells in B
            let sees_all_a = z_cells_a.iter().all(|&(zr, zc)| r == zr || c == zc);
            let sees_all_b = z_cells_b.iter().all(|&(zr, zc)| r == zr || c == zc);
            if sees_all_a && sees_all_b {
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: z,
                });
            }
        }
    }

    if actions.is_empty() {
        return TechniqueResult::NoProgress;
    }

    for action in &actions {
        if let Action::Eliminate { row, col, value } = action {
            if !state.eliminate(*row, *col, *value) {
                return TechniqueResult::Contradiction;
            }
        }
    }

    TechniqueResult::Progress(Step {
        technique: Technique::AlsXz,
        actions,
        reason: Reason::AlsXzElimination {
            als_a: a.cells.clone(),
            als_b: b.cells.clone(),
            rcc_value,
            eliminated_value: z,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::candidates::Candidates;

    #[test]
    fn no_progress_on_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        // All cells have 4 candidates, every subset is a locked set, not ALS
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn finds_als_xz_pattern() {
        // 5x5 board with manually set candidates to create an ALS-XZ pattern.
        //
        // Row 0: cells (0,0)={1,2,3} (0,1)={1,2} → ALS A in row 0:
        //   2 cells with 3 values {1,2,3}
        //
        // Col 2: cells (1,2)={2,3,4} (2,2)={3,4} → ALS B in col 2:
        //   2 cells with 3 values {2,3,4}
        //
        // Common values: {2, 3}
        // RCC = 3: cell (0,0) has 3 in row 0, cells (1,2) and (2,2) have 3 in col 2.
        //   (0,0) sees (1,2)? No (different row and column). Hmm, need to fix.
        //
        // Let me rethink: ALS A in row 0, ALS B in col 0.
        // Row 0: (0,1)={1,2,3}, (0,2)={1,3} → ALS: 2 cells, values {1,2,3}
        // Col 0: (1,0)={2,3,4}, (2,0)={2,4} → ALS: 2 cells, values {2,3,4}
        // Common: {2, 3}
        // RCC = 3: 3-cells in A = {(0,1)}, 3-cells in B = {(1,0)}.
        //   (0,1) sees (1,0)? No (row 0 vs row 1, col 1 vs col 0). Not restricted.
        //
        // For RCC, we need x-cells in A and B to share a line.
        // Easiest: A in row r, B in col c, and the RCC x only appears at the
        // intersection cell area.
        //
        // ALS A in row 2: (2,0)={1,2,3}, (2,1)={1,2} → values {1,2,3}
        // ALS B in col 0: (0,0)={3,4,5}, (1,0)={3,5} → values {3,4,5}
        // Common: {3}. But we need at least 2 common values for RCC + elimination.
        //
        // Let me try a simpler setup:
        // ALS A in row 0: (0,0)={1,2,3}, (0,1)={2,3} → values {1,2,3}
        // ALS B in row 1: (1,0)={2,3,4}, (1,1)={3,4} → values {2,3,4}
        // Common: {2,3}
        // RCC = 2: 2-cells in A = {(0,0),(0,1)}, 2-cells in B = {(1,0)}.
        //   (0,0) sees (1,0) via col 0. (0,1) sees (1,0)? No (col 1 vs col 0).
        //   Not all see each other → not restricted.
        // RCC = 2: only (0,0) and (1,0) → but we need ALL 2-cells in A to see all in B.
        //
        // Simpler: make the RCC value appear in only one cell per ALS.
        // ALS A in row 0: (0,0)={1,2,3}, (0,1)={1,2} → values {1,2,3}, 3 only in (0,0)
        // ALS B in col 0: (1,0)={3,4,5}, (2,0)={4,5} → values {3,4,5}, 3 only in (1,0)
        // Common: {3}. Only 1 common value, need 2.
        //
        // ALS A in row 0: (0,0)={1,3,4}, (0,1)={1,3} → values {1,3,4}, 4 only in (0,0)
        // ALS B in col 0: (1,0)={2,3,4}, (2,0)={2,3} → values {2,3,4}, 4 only in (1,0)
        // Common: {3, 4}
        // RCC = 4: 4-cells in A = {(0,0)}, 4-cells in B = {(1,0)}.
        //   (0,0) sees (1,0) via col 0. ✓ Restricted!
        // z = 3: 3-cells in A = {(0,0),(0,1)}, 3-cells in B = {(1,0),(2,0)}.
        //   Eliminate 3 from cells seeing all of {(0,0),(0,1)} and all of {(1,0),(2,0)}.
        //   Seeing (0,0) and (0,1) → must be in row 0 (or in both col 0 and col 1).
        //   Seeing (1,0) and (2,0) → must be in col 0 (or in both row 1 and row 2).
        //   Row 0 + col 0 → (0,0) but that's in ALS A. No external eliminations.
        //
        // Need z-cells concentrated. Let me try:
        // ALS A in row 0: (0,0)={1,3,4}, (0,1)={1,4} → values {1,3,4}
        // ALS B in col 0: (1,0)={2,3,4}, (2,0)={2,4} → values {2,3,4}
        // Common: {3, 4}
        // RCC = 3: 3-cells in A = {(0,0)}, 3-cells in B = {(1,0)}.
        //   (0,0) sees (1,0) via col 0. ✓
        // z = 4: 4-cells in A = {(0,0),(0,1)}, 4-cells in B = {(1,0),(2,0)}.
        //   Same problem as above.
        //
        // RCC = 4: 4-cells in A = {(0,0),(0,1)}, not all see 4-cells in B.
        //   Not restricted.
        //
        // Let me try with z appearing in only one cell per ALS:
        // ALS A in row 0: (0,0)={1,2,3}, (0,1)={1,2} → values {1,2,3}, 3 in (0,0) only
        // ALS B in col 0: (1,0)={2,3,4}, (2,0)={2,4} → values {2,3,4}, 3 in (1,0) only
        // Common: {2, 3}
        // RCC = 2: 2-cells in A = {(0,0),(0,1)}, 2-cells in B = {(1,0),(2,0)}.
        //   (0,1) does not see (1,0) or (2,0). Not restricted.
        // RCC = 3: 3-cells in A = {(0,0)}, 3-cells in B = {(1,0)}.
        //   (0,0) sees (1,0) via col 0. ✓
        // z = 2: 2-cells in A = {(0,0),(0,1)}, 2-cells in B = {(1,0),(2,0)}.
        //   Need to see all of them. Not practical with single cell.
        //
        // OK this is getting complex for a unit test. Let me use a larger grid
        // and set it up so z is in one cell per ALS.
        //
        // 5x5 grid:
        // ALS A in row 0: (0,0)={1,3}, (0,1)={1,4}, (0,2)={3,4} → 3 cells, values {1,3,4}
        // ALS B in col 0: (1,0)={2,3}, (2,0)={2,5}, (3,0)={3,5} → 3 cells, values {2,3,5}
        // Common: {3}. Only 1. Need 2.
        //
        // I'll just verify with a concrete 6x6 setup.
        // ALS A in row 0: (0,2)={1,2,4}, (0,3)={1,4} → 2 cells, values {1,2,4}
        // ALS B in col 2: (3,2)={2,3,5}, (4,2)={3,5} → 2 cells, values {2,3,5}
        // Common: {2}. Only 1.
        //
        // This is hard to construct by hand. Let me just verify collect_als works
        // and test the full pipeline on a real puzzle.

        let board = Board::new_empty(5);
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        let n = 5;

        // Set up ALS A in row 0: (0,0)={1,3}, (0,1)={1,2} → 2 cells, values {1,2,3}
        state.candidates[0] = Candidates::single(1).union(Candidates::single(3));
        state.candidates[1] = Candidates::single(1).union(Candidates::single(2));

        // Set up ALS B in col 0: (1,0)={3,4}, (2,0)={2,4} → 2 cells, values {2,3,4}
        state.candidates[1 * n + 0] = Candidates::single(3).union(Candidates::single(4));
        state.candidates[2 * n + 0] = Candidates::single(2).union(Candidates::single(4));

        // Common values: {2, 3}
        // RCC = 3: 3-cells in A = {(0,0)}, 3-cells in B = {(1,0)}.
        //   (0,0) sees (1,0) via col 0. ✓
        // z = 2: 2-cells in A = {(0,1)}, 2-cells in B = {(2,0)}.
        //   Cell must see (0,1) (row 0 or col 1) AND (2,0) (row 2 or col 0).
        //   (0,0) is in ALS A, skip.
        //   (2,1) → row 2 sees (2,0) ✓, col 1 sees (0,1) ✓. If has candidate 2 → eliminate!

        // Make sure (2,1) has candidate 2
        assert!(state.candidates[2 * n + 1].contains(2));

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::AlsXz);
                // Should eliminate 2 from (2,1) at minimum
                let has_target = step.actions.iter().any(|a| {
                    matches!(a, Action::Eliminate { row: 2, col: 1, value: 2 })
                });
                assert!(has_target, "Expected elimination of 2 from (2,1), got: {:?}", step.actions);
            }
            _ => panic!("Expected ALS-XZ to find a pattern"),
        }
    }
}
