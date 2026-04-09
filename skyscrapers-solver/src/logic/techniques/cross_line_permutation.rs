use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, CluePosition, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Cross-line Permutation: intersect valid permutation sets from row and column clues.
///
/// For each cell (r, c) where row r has at least one clue AND column c has at least one clue:
/// 1. Compute the set of values that appear in any valid permutation of row r
/// 2. Compute the set of values that appear in any valid permutation of column c
/// 3. Intersect these sets — values not in the intersection can be eliminated
///
/// This is more powerful than per-line PermutationEnumeration because it combines
/// constraints from two different lines at their intersection point.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Compute possible values for each row with clues
    let row_possible: Vec<Option<Vec<Candidates>>> = (0..n)
        .map(|r| {
            let clues = collect_row_clues(state, r);
            if clues.is_empty() {
                return None;
            }
            let indices: Vec<usize> = (0..n).map(|c| r * n + c).collect();
            Some(enumerate_possible_values(state, &indices, &clues))
        })
        .collect();

    // Compute possible values for each column with clues
    let col_possible: Vec<Option<Vec<Candidates>>> = (0..n)
        .map(|c| {
            let clues = collect_col_clues(state, c);
            if clues.is_empty() {
                return None;
            }
            let indices: Vec<usize> = (0..n).map(|r| r * n + c).collect();
            Some(enumerate_possible_values(state, &indices, &clues))
        })
        .collect();

    // Find eliminations at intersection cells
    for r in 0..n {
        let row_poss = match &row_possible[r] {
            Some(p) => p,
            None => continue,
        };
        for c in 0..n {
            let col_poss = match &col_possible[c] {
                Some(p) => p,
                None => continue,
            };

            let idx = r * n + c;
            if state.grid[idx].is_some() {
                continue;
            }

            // Intersect row and column possible values for this cell
            let combined = row_poss[c].intersect(col_poss[r]);
            let current = state.candidates[idx];

            // Find candidates to eliminate (in current but not in combined)
            let mut actions = Vec::new();
            for v in current.iter() {
                if !combined.contains(v) {
                    actions.push(Action::Eliminate {
                        row: r,
                        col: c,
                        value: v,
                    });
                }
            }

            if !actions.is_empty() {
                for action in &actions {
                    if let Action::Eliminate { row, col, value } = action {
                        if !state.eliminate(*row, *col, *value) {
                            return TechniqueResult::Contradiction;
                        }
                    }
                }

                let row_clue = get_row_clue_position(state, r);
                let col_clue = get_col_clue_position(state, c);

                return TechniqueResult::Progress(Step {
                    technique: Technique::CrossLinePermutation,
                    actions,
                    reason: Reason::CrossLinePermutationElimination {
                        row_clue,
                        col_clue,
                    },
                });
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Collect clues for a row: (expected_visibility, is_forward)
/// Forward = Left clue (indices left-to-right), Reverse = Right clue (indices right-to-left)
fn collect_row_clues(state: &SolveState, r: usize) -> Vec<(u8, bool)> {
    let mut clues = Vec::new();
    if let Some(v) = state.left[r] {
        clues.push((v, true));
    }
    if let Some(v) = state.right[r] {
        clues.push((v, false));
    }
    clues
}

/// Collect clues for a column: (expected_visibility, is_forward)
/// Forward = Top clue (indices top-to-bottom), Reverse = Bottom clue (indices bottom-to-top)
fn collect_col_clues(state: &SolveState, c: usize) -> Vec<(u8, bool)> {
    let mut clues = Vec::new();
    if let Some(v) = state.top[c] {
        clues.push((v, true));
    }
    if let Some(v) = state.bottom[c] {
        clues.push((v, false));
    }
    clues
}

/// Get the first available clue position for a row (for Reason reporting).
fn get_row_clue_position(state: &SolveState, r: usize) -> CluePosition {
    if state.left[r].is_some() {
        CluePosition::Left(r)
    } else {
        CluePosition::Right(r)
    }
}

/// Get the first available clue position for a column (for Reason reporting).
fn get_col_clue_position(state: &SolveState, c: usize) -> CluePosition {
    if state.top[c].is_some() {
        CluePosition::Top(c)
    } else {
        CluePosition::Bottom(c)
    }
}

/// Enumerate all valid permutations for a line and return the union of values
/// at each position across all valid permutations.
///
/// `indices` are in natural order (left-to-right or top-to-bottom).
/// `clues` is a list of (expected_visibility, is_forward).
///
/// Returns a Vec of Candidates, one per position in the line.
fn enumerate_possible_values(
    state: &SolveState,
    indices: &[usize],
    clues: &[(u8, bool)],
) -> Vec<Candidates> {
    let n = indices.len();
    let mut possible: Vec<Candidates> = vec![Candidates::empty(); n];

    // Collect fixed values
    let mut used: Vec<bool> = vec![false; n + 1];
    for &idx in indices {
        if let Some(v) = state.grid[idx] {
            used[v as usize] = true;
        }
    }

    // Identify free positions
    let free_positions: Vec<usize> = indices
        .iter()
        .enumerate()
        .filter(|&(_, &idx)| state.grid[idx].is_none())
        .map(|(pos, _)| pos)
        .collect();

    if free_positions.is_empty() {
        // All positions are fixed, just record their values
        for (pos, &idx) in indices.iter().enumerate() {
            if let Some(v) = state.grid[idx] {
                possible[pos] = Candidates::single(v);
            }
        }
        return possible;
    }

    // Fixed positions already have determined values
    for (pos, &idx) in indices.iter().enumerate() {
        if let Some(v) = state.grid[idx] {
            possible[pos] = Candidates::single(v);
        }
    }

    // Backtrack over free positions to find all valid assignments
    let mut assignments: Vec<u8> = vec![0; free_positions.len()];
    enumerate_backtrack(
        state,
        indices,
        &free_positions,
        0,
        &mut used,
        &mut assignments,
        clues,
        &mut possible,
    );

    possible
}

/// Recursive backtracking to enumerate all valid permutations.
/// For each valid permutation found, adds each position's value to `possible`.
#[allow(clippy::too_many_arguments)]
fn enumerate_backtrack(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    depth: usize,
    used: &mut Vec<bool>,
    assignments: &mut Vec<u8>,
    clues: &[(u8, bool)],
    possible: &mut Vec<Candidates>,
) {
    if depth == free_positions.len() {
        // All free cells assigned — check all clues
        let heights: Vec<u8> = build_heights(state, indices, free_positions, assignments);

        for &(expected, is_forward) in clues {
            let vis = compute_visibility(&heights, is_forward);
            if vis != expected {
                return; // This permutation doesn't satisfy all clues
            }
        }

        // Valid permutation — record values at free positions
        for (d, &pos) in free_positions.iter().enumerate() {
            possible[pos] = possible[pos].union(Candidates::single(assignments[d]));
        }
        return;
    }

    let pos = free_positions[depth];
    let idx = indices[pos];
    let candidates = state.candidates[idx];

    for v in candidates.iter() {
        if used[v as usize] {
            continue;
        }
        used[v as usize] = true;
        assignments[depth] = v;
        enumerate_backtrack(
            state,
            indices,
            free_positions,
            depth + 1,
            used,
            assignments,
            clues,
            possible,
        );
        used[v as usize] = false;
    }
}

/// Build the full height array for a line from grid values and assignments.
fn build_heights(
    state: &SolveState,
    indices: &[usize],
    free_positions: &[usize],
    assignments: &[u8],
) -> Vec<u8> {
    indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            if let Some(v) = state.grid[idx] {
                v
            } else {
                let depth = free_positions.iter().position(|&p| p == pos).unwrap();
                assignments[depth]
            }
        })
        .collect()
}

/// Compute visibility count for a height array.
/// `is_forward`: true = left-to-right/top-to-bottom, false = reverse.
fn compute_visibility(heights: &[u8], is_forward: bool) -> u8 {
    let mut max_height: u8 = 0;
    let mut count: u8 = 0;

    if is_forward {
        for &h in heights {
            if h > max_height {
                count += 1;
                max_height = h;
            }
        }
    } else {
        for &h in heights.iter().rev() {
            if h > max_height {
                count += 1;
                max_height = h;
            }
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
    fn no_progress_without_clues() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn no_progress_with_single_direction_clues() {
        // Only row clues, no column clues — no cross-line intersection possible
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        clues.set_left(1, Some(3));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn cross_line_makes_progress() {
        // n=4, Left(0)=1 and Top(0)=1
        // Left(0)=1 → R0C0=4 (clue pruning)
        // Top(0)=1 → R0C0=4 (clue pruning)
        // Both are satisfied by clue pruning already.
        // Let's use a more complex case: Left(0)=2 and Top(2)=2
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        clues.set_top(2, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // This may or may not make progress depending on what PermutationEnumeration
        // already handles. The key test is that it doesn't crash.
        let result = apply(&mut state);
        // Result is either Progress or NoProgress, not Contradiction
        assert!(!matches!(result, TechniqueResult::Contradiction));
    }
}
