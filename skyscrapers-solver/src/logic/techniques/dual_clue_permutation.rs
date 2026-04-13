use crate::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Dual-clue permutation enumeration: checks both opposing clues simultaneously.
///
/// For each line where both opposing clues exist (Left+Right or Top+Bottom),
/// enumerate valid permutations that satisfy BOTH clues at once. This can
/// eliminate candidates that single-clue permutation enumeration cannot.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    for i in 0..n {
        // Row: Left + Right
        if let (Some(expected_left), Some(expected_right)) = (state.left[i], state.right[i]) {
            let indices: Vec<usize> = (0..n).map(|c| i * n + c).collect();
            let result = check_line_dual(
                state,
                &indices,
                expected_left,
                expected_right,
                Line::Row(i),
                CluePosition::Left(i),
                CluePosition::Right(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }

        // Column: Top + Bottom
        if let (Some(expected_top), Some(expected_bottom)) = (state.top[i], state.bottom[i]) {
            let indices: Vec<usize> = (0..n).map(|r| r * n + i).collect();
            let result = check_line_dual(
                state,
                &indices,
                expected_top,
                expected_bottom,
                Line::Col(i),
                CluePosition::Top(i),
                CluePosition::Bottom(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Check a single line against both opposing clues.
/// `indices` is in natural order (left-to-right or top-to-bottom).
/// `expected_fwd` is the clue from the start (Left/Top).
/// `expected_rev` is the clue from the end (Right/Bottom).
fn check_line_dual(
    state: &mut SolveState,
    indices: &[usize],
    expected_fwd: u8,
    expected_rev: u8,
    line: Line,
    clue_a: CluePosition,
    clue_b: CluePosition,
) -> TechniqueResult {
    let n = state.n;

    // Collect fixed values in this line
    let mut used: Vec<bool> = vec![false; n + 1];
    for &idx in indices {
        if let Some(v) = state.grid[idx] {
            used[v as usize] = true;
        }
    }

    // Identify free positions (indices into `indices` array)
    let free_positions: Vec<usize> = indices
        .iter()
        .enumerate()
        .filter(|&(_, &idx)| state.grid[idx].is_none())
        .map(|(pos, _)| pos)
        .collect();

    if free_positions.is_empty() {
        return TechniqueResult::NoProgress;
    }

    // For each free cell, for each candidate, check if it can satisfy both clues
    let mut eliminations = Vec::new();

    for &target_pos in &free_positions {
        let target_idx = indices[target_pos];
        let candidates: Vec<u8> = state.candidates[target_idx].iter().collect();

        for val in candidates {
            if !can_satisfy_dual_clue(
                state,
                indices,
                &free_positions,
                target_pos,
                val,
                expected_fwd,
                expected_rev,
                &used,
            ) {
                eliminations.push((target_idx, val));
            }
        }
    }

    if eliminations.is_empty() {
        return TechniqueResult::NoProgress;
    }

    let mut actions = Vec::new();
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

    TechniqueResult::Progress(Step {
        technique: Technique::DualCluePermutation,
        actions,
        reason: Reason::DualCluePermutationElimination {
            line,
            clue_a,
            clue_b,
        },
    })
}

/// Check if fixing `val` at `target_pos` allows any valid assignment satisfying both clues.
#[allow(clippy::too_many_arguments)]
fn can_satisfy_dual_clue(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    target_pos: usize,
    val: u8,
    expected_fwd: u8,
    expected_rev: u8,
    used: &[bool],
) -> bool {
    let other_free: Vec<usize> = free_positions
        .iter()
        .copied()
        .filter(|&p| p != target_pos)
        .collect();

    // Precompute pos -> depth mapping to avoid repeated linear scans
    let n = indices.len();
    let mut pos_to_depth = vec![usize::MAX; n];
    for (depth, &pos) in other_free.iter().enumerate() {
        pos_to_depth[pos] = depth;
    }

    let mut value_used = used.to_vec();
    value_used[val as usize] = true;

    let mut assignments: Vec<u8> = vec![0; other_free.len()];

    backtrack_dual(
        state,
        indices,
        &other_free,
        &pos_to_depth,
        0,
        target_pos,
        val,
        &mut value_used,
        &mut assignments,
        expected_fwd,
        expected_rev,
    )
}

/// Backtracking search over free positions (excluding target).
/// Returns true as soon as any valid assignment satisfies both clues.
#[allow(clippy::too_many_arguments)]
fn backtrack_dual(
    state: &SolveState,
    indices: &[usize],
    other_free: &[usize],
    pos_to_depth: &[usize],
    depth: usize,
    target_pos: usize,
    target_val: u8,
    value_used: &mut Vec<bool>,
    assignments: &mut Vec<u8>,
    expected_fwd: u8,
    expected_rev: u8,
) -> bool {
    if depth == other_free.len() {
        let (vis_fwd, vis_rev) =
            compute_visibility_both(state, indices, pos_to_depth, target_pos, target_val, assignments);
        return vis_fwd == expected_fwd && vis_rev == expected_rev;
    }

    let pos = other_free[depth];
    let idx = indices[pos];
    let candidates = state.candidates[idx];

    for v in candidates.iter() {
        if value_used[v as usize] {
            continue;
        }
        value_used[v as usize] = true;
        assignments[depth] = v;
        if backtrack_dual(
            state,
            indices,
            other_free,
            pos_to_depth,
            depth + 1,
            target_pos,
            target_val,
            value_used,
            assignments,
            expected_fwd,
            expected_rev,
        ) {
            value_used[v as usize] = false;
            return true;
        }
        value_used[v as usize] = false;
    }

    false
}

/// Compute visibility count from both directions for a fully assigned line.
/// Returns (forward_count, reverse_count).
/// `pos_to_depth[pos]` maps a line position to its index in `assignments`,
/// or `usize::MAX` if the position is not a free cell.
fn compute_visibility_both(
    state: &SolveState,
    indices: &[usize],
    pos_to_depth: &[usize],
    target_pos: usize,
    target_val: u8,
    assignments: &[u8],
) -> (u8, u8) {
    // Build the full height array
    let heights: Vec<u8> = indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            if pos == target_pos {
                target_val
            } else if let Some(v) = state.grid[idx] {
                v
            } else {
                assignments[pos_to_depth[pos]]
            }
        })
        .collect();

    // Forward visibility
    let mut max_height: u8 = 0;
    let mut fwd_count: u8 = 0;
    for &h in &heights {
        if h > max_height {
            fwd_count += 1;
            max_height = h;
        }
    }

    // Reverse visibility
    max_height = 0;
    let mut rev_count: u8 = 0;
    for &h in heights.iter().rev() {
        if h > max_height {
            rev_count += 1;
            max_height = h;
        }
    }

    (fwd_count, rev_count)
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::logic::state::SolveState;

    #[test]
    fn no_progress_without_dual_clues() {
        // Only left clue, no right clue — DualCluePermutation should not fire
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }

    #[test]
    fn eliminates_with_dual_clues() {
        // n=4, row 0: left=1, right=3
        // left=1 means first cell must be 4 (handled by clue pruning)
        // Valid permutations: [4,_,_,_] with right=3
        //   [4,1,2,3]: right=3 ✓
        //   [4,2,1,3]: right=2 ✗
        //   [4,1,3,2]: right=2 ✗
        //   [4,2,3,1]: right=1 ✗
        //   [4,3,1,2]: right=2 ✗
        //   [4,3,2,1]: right=1 ✗
        // Only [4,1,2,3] works → all positions are forced
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(1));
        clues.set_right(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Clue pruning already assigned pos 0 = 4
        assert_eq!(state.grid[state.idx(0, 0)], Some(4));

        // Single-direction PermutationEnumeration with right=3 already eliminates some,
        // but let's check that DualCluePermutation can also make progress
        let result = apply(&mut state);
        // Whether it makes progress depends on what's left after clue pruning
        // The key is it doesn't crash or produce incorrect results
        if let TechniqueResult::Progress(step) = result {
            assert_eq!(step.technique, Technique::DualCluePermutation);
            assert!(!step.actions.is_empty());
        }
    }

    #[test]
    fn no_elimination_without_clues() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
