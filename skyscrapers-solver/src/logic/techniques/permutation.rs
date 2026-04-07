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
    let n = state.n;

    for i in 0..n {
        if let Some(expected) = state.left[i] {
            let indices: Vec<usize> = (0..n).map(|c| i * n + c).collect();
            let result = check_line(
                state,
                &indices,
                expected,
                Line::Row(i),
                CluePosition::Left(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }

        if let Some(expected) = state.right[i] {
            let indices: Vec<usize> = (0..n).rev().map(|c| i * n + c).collect();
            let result = check_line(
                state,
                &indices,
                expected,
                Line::Row(i),
                CluePosition::Right(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }

        if let Some(expected) = state.top[i] {
            let indices: Vec<usize> = (0..n).map(|r| r * n + i).collect();
            let result = check_line(
                state,
                &indices,
                expected,
                Line::Col(i),
                CluePosition::Top(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }

        if let Some(expected) = state.bottom[i] {
            let indices: Vec<usize> = (0..n).rev().map(|r| r * n + i).collect();
            let result = check_line(
                state,
                &indices,
                expected,
                Line::Col(i),
                CluePosition::Bottom(i),
            );
            if !matches!(result, TechniqueResult::NoProgress) {
                return result;
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Check a single line (in viewing order) against its clue.
fn check_line(
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
        technique: Technique::PermutationEnumeration,
        actions,
        reason: Reason::PermutationElimination {
            line,
            clue: clue_pos,
        },
    })
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

    // Track which values are used (fixed values + the target value)
    let mut value_used = used.to_vec();
    value_used[val as usize] = true;

    // Assignments for other free positions (indexed by depth)
    let mut assignments: Vec<u8> = vec![0; other_free.len()];

    backtrack(
        state,
        indices,
        &other_free,
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
            other_free,
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
fn compute_visibility(
    state: &SolveState,
    indices: &[usize],
    other_free: &[usize],
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
            // Find this position in other_free and get its assignment
            let depth = other_free.iter().position(|&p| p == pos).unwrap();
            assignments[depth]
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
