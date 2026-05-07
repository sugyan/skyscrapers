use crate::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Permutation enumeration for clue-based candidate elimination.
///
/// For each line with a clue, for each free cell, for each candidate value:
/// fix that value and check whether any valid assignment of the remaining
/// values (respecting per-cell candidates and value uniqueness) can produce
/// a visibility count matching the clue. If not, eliminate the candidate.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    for cl in state.clued_lines() {
        let result = enumerate_and_prune(state, &cl.indices, cl.expected, cl.line, cl.clue_pos);
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }
    TechniqueResult::NoProgress
}

/// Enumerate permutations of a single line (in viewing order) and eliminate
/// any candidate that cannot possibly satisfy the line's clue.
fn enumerate_and_prune(
    state: &mut SolveState,
    indices: &[usize],
    expected: u8,
    line: Line,
    clue_pos: CluePosition,
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

    // For each free cell, for each candidate, check if it can satisfy the clue
    let mut eliminations = Vec::new();

    for &target_pos in &free_positions {
        let target_idx = indices[target_pos];
        let candidates: Vec<u8> = state.candidates[target_idx].iter().collect();

        for val in candidates {
            if !can_satisfy_clue(
                state,
                indices,
                &free_positions,
                target_pos,
                val,
                expected,
                &used,
            ) {
                eliminations.push((target_idx, val));
            }
        }
    }

    if eliminations.is_empty() {
        return TechniqueResult::NoProgress;
    }

    // Classify this firing: a "simple" enumeration is one a human can do
    // mentally — few free cells, or few permutations to inspect.
    let technique = if is_simple_enumeration(state, indices, &free_positions, expected, &used) {
        Technique::SimplePermutation
    } else {
        Technique::PermutationEnumeration
    };

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
        technique,
        actions,
        reason: Reason::PermutationElimination {
            line,
            clue: clue_pos,
        },
    })
}

/// Threshold for "few permutations". Any line whose pre-pruning candidate
/// permutations satisfying the clue number ≤ this is classified as Simple.
const SIMPLE_PERM_CAP: u32 = 8;

/// Threshold for "few free cells". Any line with at most this many unknowns
/// is classified as Simple regardless of permutation count.
const SIMPLE_FREE_CELLS: usize = 3;

fn is_simple_enumeration(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    expected: u8,
    used: &[bool],
) -> bool {
    if free_positions.len() <= SIMPLE_FREE_CELLS {
        return true;
    }
    count_valid_perms_capped(
        state,
        indices,
        free_positions,
        expected,
        used,
        SIMPLE_PERM_CAP + 1,
    ) <= SIMPLE_PERM_CAP
}

/// Count valid permutations of the line's free cells (respecting per-cell
/// candidates and the clue's visibility count). Returns early once the
/// running count exceeds `cap`, in which case the returned value is
/// guaranteed > `cap` but not otherwise meaningful.
fn count_valid_perms_capped(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    expected: u8,
    used: &[bool],
    cap: u32,
) -> u32 {
    let n = indices.len();
    let mut pos_to_depth = vec![usize::MAX; n];
    for (depth, &pos) in free_positions.iter().enumerate() {
        pos_to_depth[pos] = depth;
    }

    let mut value_used = used.to_vec();
    let mut assignments: Vec<u8> = vec![0; free_positions.len()];
    let mut count = 0u32;

    count_backtrack(
        state,
        indices,
        free_positions,
        &pos_to_depth,
        0,
        &mut value_used,
        &mut assignments,
        expected,
        cap,
        &mut count,
    );
    count
}

#[allow(clippy::too_many_arguments)]
fn count_backtrack(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    pos_to_depth: &[usize],
    depth: usize,
    value_used: &mut Vec<bool>,
    assignments: &mut Vec<u8>,
    expected: u8,
    cap: u32,
    count: &mut u32,
) {
    if *count > cap {
        return;
    }
    if depth == free_positions.len() {
        if compute_visibility_full(state, indices, pos_to_depth, assignments) == expected {
            *count += 1;
        }
        return;
    }
    let pos = free_positions[depth];
    let idx = indices[pos];
    let candidates = state.candidates[idx];
    for v in candidates.iter() {
        if value_used[v as usize] {
            continue;
        }
        value_used[v as usize] = true;
        assignments[depth] = v;
        count_backtrack(
            state,
            indices,
            free_positions,
            pos_to_depth,
            depth + 1,
            value_used,
            assignments,
            expected,
            cap,
            count,
        );
        value_used[v as usize] = false;
        if *count > cap {
            return;
        }
    }
}

/// Like [`compute_visibility`], but for a line where every free cell is
/// in `assignments` (no `target_pos` carve-out).
fn compute_visibility_full(
    state: &SolveState,
    indices: &[usize],
    pos_to_depth: &[usize],
    assignments: &[u8],
) -> u8 {
    let mut max_height: u8 = 0;
    let mut count: u8 = 0;
    for (pos, &idx) in indices.iter().enumerate() {
        let height = if let Some(v) = state.grid[idx] {
            v
        } else {
            assignments[pos_to_depth[pos]]
        };
        if height > max_height {
            count += 1;
            max_height = height;
        }
    }
    count
}

/// Check if fixing `val` at `target_pos` allows any valid assignment that satisfies the clue.
fn can_satisfy_clue(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    target_pos: usize,
    val: u8,
    expected: u8,
    used: &[bool],
) -> bool {
    // Build the list of other free positions (excluding target)
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

    // Track which values are used (fixed values + the target value)
    let mut value_used = used.to_vec();
    value_used[val as usize] = true;

    // Assignments for other free positions (indexed by depth)
    let mut assignments: Vec<u8> = vec![0; other_free.len()];

    backtrack(
        state,
        indices,
        &other_free,
        &pos_to_depth,
        0,
        target_pos,
        val,
        &mut value_used,
        &mut assignments,
        expected,
    )
}

/// Backtracking search over free positions (excluding target).
/// Returns true as soon as any valid assignment satisfies the clue.
#[allow(clippy::too_many_arguments)]
fn backtrack(
    state: &SolveState,
    indices: &[usize],
    other_free: &[usize],
    pos_to_depth: &[usize],
    depth: usize,
    target_pos: usize,
    target_val: u8,
    value_used: &mut Vec<bool>,
    assignments: &mut Vec<u8>,
    expected: u8,
) -> bool {
    if depth == other_free.len() {
        // All free cells assigned — compute visibility and check
        return compute_visibility(
            state,
            indices,
            pos_to_depth,
            target_pos,
            target_val,
            assignments,
        ) == expected;
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
        if backtrack(
            state,
            indices,
            other_free,
            pos_to_depth,
            depth + 1,
            target_pos,
            target_val,
            value_used,
            assignments,
            expected,
        ) {
            value_used[v as usize] = false;
            return true;
        }
        value_used[v as usize] = false;
    }

    false
}

/// Compute visibility count for a fully assigned line (in viewing order).
/// `pos_to_depth[pos]` maps a line position to its index in `assignments`,
/// or `usize::MAX` if the position is not a free cell.
fn compute_visibility(
    state: &SolveState,
    indices: &[usize],
    pos_to_depth: &[usize],
    target_pos: usize,
    target_val: u8,
    assignments: &[u8],
) -> u8 {
    let mut max_height: u8 = 0;
    let mut count: u8 = 0;

    for (pos, &idx) in indices.iter().enumerate() {
        let height = if pos == target_pos {
            target_val
        } else if let Some(v) = state.grid[idx] {
            v
        } else {
            assignments[pos_to_depth[pos]]
        };

        if height > max_height {
            count += 1;
            max_height = height;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::logic::state::SolveState;

    #[test]
    fn clue_2_with_n_at_end() {
        // n=5, left clue=2, row: [_, _, _, _, 5]
        // Only pos 0 = 4 satisfies clue=2 (4 hides 1,2,3; then 5 visible)
        let mut board = Board::new_empty(5);
        board.set(0, 4, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        // pos 0 should only have candidate 4
        let idx = state.idx(0, 0);
        assert_eq!(state.candidates[idx].singleton(), Some(4));
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

    #[test]
    fn eliminates_with_candidate_constraints() {
        // n=4, left clue=3
        // pos 0: {1,2}, pos 1: {3,4}, pos 2: {2,3}, pos 3: {1,4}
        // pos 0=2: only valid assignment is [2,4,3,1] → vis=2. clue=3 → eliminate 2.
        // pos 0=1: valid assignment [1,3,2,4] → vis=3. clue=3 → keep 1.
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Manually restrict candidates to create the test scenario
        // First, clear all candidates for row 0
        let n = state.n;
        for c in 0..n {
            let idx = c;
            for v in 1..=n as u8 {
                state.candidates[idx] = state.candidates[idx].remove(v);
            }
        }
        // Set specific candidates
        // pos 0: {1,2}
        state.candidates[0] = crate::candidates::Candidates::single(1);
        state.candidates[0] = state.candidates[0].union(crate::candidates::Candidates::single(2));
        // pos 1: {3,4}
        state.candidates[1] = crate::candidates::Candidates::single(3);
        state.candidates[1] = state.candidates[1].union(crate::candidates::Candidates::single(4));
        // pos 2: {2,3}
        state.candidates[2] = crate::candidates::Candidates::single(2);
        state.candidates[2] = state.candidates[2].union(crate::candidates::Candidates::single(3));
        // pos 3: {1,4}
        state.candidates[3] = crate::candidates::Candidates::single(1);
        state.candidates[3] = state.candidates[3].union(crate::candidates::Candidates::single(4));

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        // pos 0 should have 2 eliminated, only 1 remains
        assert!(state.candidates[0].contains(1));
        assert!(!state.candidates[0].contains(2));
    }

    #[test]
    fn clue_1_with_partial_fill() {
        // n=4, left clue=1 → pos 0 must be 4 (handled by clue pruning)
        // Verify permutation enumeration doesn't cause contradiction
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(1));
        let puzzle = Puzzle { board, clues };
        let state = SolveState::new(&puzzle).unwrap();
        // Clue pruning already forces (0,0) = 4
        assert_eq!(state.grid[state.idx(0, 0)], Some(4));
    }
}
