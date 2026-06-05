use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Cap on the number of *free* cells in the prefix being enumerated. Keeps the
/// per-target search bounded (≤ `PREFIX_CAP!` arrangements) and limits the
/// technique to the short, humanly-tractable forward deductions. Targets whose
/// prefix has more free cells are left for the full permutation techniques.
const PREFIX_CAP: usize = 4;

/// Forward (prefix-only) visibility pruning.
///
/// A cell's visibility from a clue's edge is decided entirely by the cells
/// *before* it in viewing order. So for a target cell, we can enumerate just
/// the prefix up to (and including) it — ignoring the suffix — and bound how
/// many additional buildings the suffix could reveal. If, for a candidate
/// value, *every* prefix arrangement makes the clue's count unreachable even
/// after accounting for that suffix slack, the candidate is eliminated.
///
/// This is strictly weaker than full [`permutation`](super::permutation)
/// enumeration (which also uses the suffix's candidates), so it never finds an
/// elimination the full technique would miss. Its purpose is to recognise the
/// cheap, forward deductions a human makes and attribute them at the lower
/// `PrefixPermutation` (Hard) tier rather than letting them be absorbed by the
/// Expert-tier permutation/ALS techniques.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    for cl in state.clued_lines() {
        let indices = &cl.indices;
        let k = cl.expected;

        // Values already fixed anywhere in the line are unavailable to the
        // free prefix cells (the line is a permutation).
        let mut used = vec![false; n + 1];
        for &idx in indices {
            if let Some(v) = state.grid[idx] {
                used[v as usize] = true;
            }
        }

        let mut eliminations: Vec<(usize, u8)> = Vec::new();

        for p in 0..indices.len() {
            let target_idx = indices[p];
            if state.grid[target_idx].is_some() {
                continue;
            }

            // Free positions within the prefix [0..=p], excluding the target.
            let free_prefix: Vec<usize> = (0..=p)
                .filter(|&pos| state.grid[indices[pos]].is_none() && pos != p)
                .collect();

            // Bound the search: only short prefixes (counting the target).
            if free_prefix.len() + 1 > PREFIX_CAP {
                continue;
            }

            let candidates: Vec<u8> = state.candidates[target_idx].iter().collect();
            for v in candidates {
                if !prefix_allows_clue(state, indices, &free_prefix, &used, p, v, k) {
                    eliminations.push((target_idx, v));
                }
            }
        }

        if eliminations.is_empty() {
            continue;
        }

        let mut actions = Vec::with_capacity(eliminations.len());
        for &(idx, v) in &eliminations {
            let r = idx / n;
            let c = idx % n;
            actions.push(Action::Eliminate {
                row: r,
                col: c,
                value: v,
            });
            if !state.eliminate(r, c, v) {
                return TechniqueResult::Contradiction;
            }
        }

        return TechniqueResult::Progress(Step {
            technique: Technique::PrefixPermutation,
            actions,
            reason: Reason::PermutationElimination {
                line: cl.line,
                clue: cl.clue_pos,
            },
        });
    }

    TechniqueResult::NoProgress
}

/// True iff *some* assignment of the prefix `[0..=p]` (with `target_val` fixed
/// at position `p`, other free prefix cells drawn from their candidates and all
/// values distinct) is consistent with the line's clue `k`, after allowing the
/// suffix to add between `lo_suf` and `hi_suf` further visible buildings.
fn prefix_allows_clue(
    state: &SolveState,
    indices: &[usize],
    free_prefix: &[usize],
    used: &[bool],
    p: usize,
    target_val: u8,
    k: u8,
) -> bool {
    let n = state.n as u8;
    let suffix_cells = (indices.len() - 1 - p) as u8;

    let mut value_used = used.to_vec();
    value_used[target_val as usize] = true;
    let mut assignment = vec![0u8; free_prefix.len()];

    search(
        state,
        indices,
        free_prefix,
        p,
        target_val,
        n,
        suffix_cells,
        k,
        0,
        &mut value_used,
        &mut assignment,
    )
}

#[allow(clippy::too_many_arguments)]
fn search(
    state: &SolveState,
    indices: &[usize],
    free_prefix: &[usize],
    p: usize,
    target_val: u8,
    n: u8,
    suffix_cells: u8,
    k: u8,
    depth: usize,
    value_used: &mut [bool],
    assignment: &mut [u8],
) -> bool {
    if depth == free_prefix.len() {
        let (vis, m) = prefix_visibility(state, indices, free_prefix, p, target_val, assignment);
        let lo_suf = if m < n { 1 } else { 0 };
        let hi_suf = (n - m).min(suffix_cells);
        return vis + lo_suf <= k && k <= vis + hi_suf;
    }

    let idx = indices[free_prefix[depth]];
    for val in state.candidates[idx].iter() {
        if value_used[val as usize] {
            continue;
        }
        value_used[val as usize] = true;
        assignment[depth] = val;
        if search(
            state,
            indices,
            free_prefix,
            p,
            target_val,
            n,
            suffix_cells,
            k,
            depth + 1,
            value_used,
            assignment,
        ) {
            value_used[val as usize] = false;
            return true;
        }
        value_used[val as usize] = false;
    }
    false
}

/// Compute `(visible_count, max_height)` over the prefix `[0..=p]` in viewing
/// order, taking fixed cells from the grid, `target_val` at position `p`, and
/// the remaining free prefix cells from `assignment`.
fn prefix_visibility(
    state: &SolveState,
    indices: &[usize],
    free_prefix: &[usize],
    p: usize,
    target_val: u8,
    assignment: &[u8],
) -> (u8, u8) {
    let mut max_height: u8 = 0;
    let mut count: u8 = 0;
    for (pos, &idx) in indices.iter().enumerate().take(p + 1) {
        let height = if pos == p {
            target_val
        } else if let Some(v) = state.grid[idx] {
            v
        } else {
            // Position is a free prefix cell — find its assigned value.
            let depth = free_prefix.iter().position(|&fp| fp == pos).unwrap();
            assignment[depth]
        };
        if height > max_height {
            count += 1;
            max_height = height;
        }
    }
    (count, max_height)
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::candidates::Candidates;
    use crate::logic::difficulty::Technique;

    #[test]
    fn clue_2_eliminates_n_minus_1_at_second_cell() {
        // n=4, Left=2 on row 0. After clue pruning R0C0 ∈ {1,2,3} (no 4).
        // The 2nd cell (R0C1) cannot be 3: any prefix [a,3] with a<3 makes
        // both cells visible plus the later 4 → ≥3 visible. Value 4 stays
        // (prefix [a,4] gives exactly 2 visible).
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        let c1 = state.candidates[state.idx(0, 1)];
        assert!(!c1.contains(3), "3 should be eliminated from R0C1");
        assert!(c1.contains(4), "4 must remain at R0C1");
    }

    #[test]
    fn emits_prefix_permutation_technique_label() {
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        match apply(&mut state) {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::PrefixPermutation);
            }
            _ => panic!("expected Progress"),
        }
    }

    #[test]
    fn clue_3_with_fixed_first_cell_prunes_forward() {
        // n=4, Left=3 on row 0, R0C0 fixed to 1. With the first (shortest)
        // building seen, two more must appear in ascending order among the
        // rest. The 2nd cell cannot be 4 (that would leave only 2 visible),
        // so 4 is eliminated from R0C1.
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(1));
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));
        assert!(
            !state.candidates[state.idx(0, 1)].contains(4),
            "4 should be eliminated from R0C1 for Left=3 with R0C0=1"
        );
    }

    #[test]
    fn keeps_valid_candidates() {
        // A line with no clue produces no eliminations.
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn does_not_eliminate_reachable_value() {
        // n=4, Left=2: the first cell may legitimately be 1, 2 or 3. Ensure
        // none of those are wrongly removed from R0C0 (only 4 is, via clue
        // pruning during `new`, not this technique).
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        let _ = apply(&mut state);
        let c0 = state.candidates[state.idx(0, 0)];
        for v in [1u8, 2, 3] {
            assert!(c0.contains(v), "{v} must remain a candidate at R0C0");
        }
        // sanity: Candidates import used
        assert_eq!(Candidates::single(1).singleton(), Some(1));
    }
}
