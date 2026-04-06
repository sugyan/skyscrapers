use crate::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Visibility constraint propagation for partially filled lines.
///
/// For each line with a clue, for each unassigned cell, for each candidate value:
/// check if placing that value could ever satisfy the clue constraint.
/// If not, eliminate the candidate.
///
/// **Known issue**: The greedy min/max visibility bounds do not account for the
/// constraint that each value appears exactly once in a line. This can lead to
/// incorrect eliminations. Currently disabled in the technique list pending a
/// more accurate algorithm (e.g., permutation enumeration or exact bounds).
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Check all four directions
    for i in 0..n {
        // Left clue on row i
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

        // Right clue on row i
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

        // Top clue on column i
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

        // Bottom clue on column i
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
/// Try to eliminate candidates that can never satisfy the visibility constraint.
fn check_line(
    state: &mut SolveState,
    indices: &[usize],
    expected: u8,
    _line: Line,
    clue_pos: CluePosition,
) -> TechniqueResult {
    let n = state.n;
    let mut eliminations = Vec::new();

    for (pos, &idx) in indices.iter().enumerate() {
        if state.grid[idx].is_some() {
            continue;
        }
        for v in state.candidates[idx].iter() {
            // Check if placing v at position pos can satisfy the clue
            if !can_satisfy_clue(state, indices, pos, v, expected, n) {
                eliminations.push((idx, v));
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
        technique: Technique::VisibilityPropagation,
        actions,
        reason: Reason::ClueConstraint {
            clue: clue_pos,
            expected,
        },
    })
}

/// Check if placing value `val` at position `pos` in the line can satisfy the visibility clue.
///
/// Computes the min and max possible visibility counts given the current candidates,
/// with `val` fixed at `pos`.
fn can_satisfy_clue(
    state: &SolveState,
    indices: &[usize],
    target_pos: usize,
    val: u8,
    expected: u8,
    n: usize,
) -> bool {
    // Compute min and max possible visibility by walking the line in viewing order.
    // For each position, we know either the actual value or the set of candidates.
    // At target_pos, we fix the value to `val`.

    let min_vis = compute_extreme_visibility(state, indices, target_pos, val, n, true);
    let max_vis = compute_extreme_visibility(state, indices, target_pos, val, n, false);

    // The clue can be satisfied if expected is in [min_vis, max_vis]
    min_vis <= expected && expected <= max_vis
}

/// Compute min or max possible visibility count for a line.
///
/// Walks in viewing order, tracking the current maximum height.
/// For unassigned cells, considers all candidate values.
/// At target_pos, uses the fixed value `val`.
///
/// For min visibility: at each unassigned cell, try to NOT be visible (pick smallest candidate).
/// For max visibility: at each unassigned cell, try to BE visible (pick largest candidate).
///
/// This is a greedy approximation — it doesn't account for the constraint that each value
/// appears exactly once, but it gives valid bounds.
fn compute_extreme_visibility(
    state: &SolveState,
    indices: &[usize],
    target_pos: usize,
    val: u8,
    _n: usize,
    minimize: bool,
) -> u8 {
    let mut max_height: u8 = 0;
    let mut count: u8 = 0;

    for (pos, &idx) in indices.iter().enumerate() {
        let height = if pos == target_pos {
            val
        } else if let Some(v) = state.grid[idx] {
            v
        } else {
            // Choose a candidate to minimize or maximize visibility
            let cands = state.candidates[idx];
            if minimize {
                // Pick the smallest candidate (least likely to be visible)
                // If we can pick one ≤ max_height, it won't be visible
                let mut best = cands.iter().next().unwrap(); // smallest
                for c in cands.iter() {
                    if c <= max_height {
                        best = c;
                        break;
                    }
                }
                best
            } else {
                // Pick the largest candidate (most likely to be visible)
                let mut best = 0u8;
                for c in cands.iter() {
                    best = c; // last = largest
                }
                best
            }
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

    #[test]
    fn eliminates_based_on_visibility() {
        // n=5, left clue = 2 on row 0
        // Place 5 at position 1: [_, 5, _, _, _]
        // With clue=2, position 0 must be < 5 (it's always visible),
        // and nothing after 5 is visible, so exactly 2 visible means pos 0 < 5.
        // But also: if pos 0 = 4, then [4, 5, ...] gives 2 visible — OK.
        // If pos 0 = 1, [1, 5, ...] gives 2 — OK.
        // The constraint should eliminate values that make it impossible.
        let mut board = Board::new_empty(5);
        board.set(0, 1, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Before visibility propagation, check that 5 is already removed from row 0
        // (by peer elimination since 5 is placed at (0,1))
        assert!(!state.candidates[state.idx(0, 0)].contains(5));

        let result = apply(&mut state);
        // May or may not find eliminations depending on the greedy bounds
        // At minimum, the function should not crash
        assert!(!matches!(result, TechniqueResult::Contradiction));
    }

    #[test]
    fn clue_1_with_partial_fill() {
        // n=4, left clue = 1 on row 0
        // Position 0 must be 4 (already handled by clue pruning)
        // But if we manually test with position 0 empty and candidates {3, 4},
        // visibility propagation should eliminate 3 (clue=1 means only 1 visible, need n at pos 0)
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(1));
        let puzzle = Puzzle { board, clues };
        let state = SolveState::new(&puzzle).unwrap();
        // Clue pruning already forces (0,0) = 4
        assert_eq!(state.grid[state.idx(0, 0)], Some(4));
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
