use crate::logic::difficulty::{Action, CluePosition, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Analytical visibility forcing: the "before-n" saturation case.
///
/// Restricts itself to one tightly-bounded observation:
///
/// * If `n` is already placed in the line at position `k` (0-indexed from
///   the viewer), then positions beyond `n` are a dead-zone and contribute
///   zero visibles, `n` itself contributes one, and the `k` positions
///   before `n` can contribute **at most** `k` visibles. So the clue must
///   satisfy `clue <= k + 1`.
///
/// * `clue - 1 == k` (and `k > 0`) → **Saturation**: every cell before `n`
///   must be a new running-max, i.e. the pre-`n` prefix is strictly
///   ascending. Each such cell is forced to be `> running_max` (the
///   largest value in strictly-earlier pre-`n` cells) and `< n`.
///
/// * Any other shape (`clue - 1 > k` or `clue - 1 < k`) → defer.
///   `clue - 1 > k` *is* a contradiction in principle, but init-time
///   clue pruning already caps each position's candidates such that no
///   later technique can ever place `n` at a position that violates the
///   clue — so this path is unreachable and left implicit rather than
///   duplicating a check that would never fire.
///
/// The technique does not examine any line where `n` is not yet placed —
/// the clue=n case is already handled by clue pruning during init, and
/// other unsaturated shapes are out of scope by design.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;
    let n_val = n as u8;

    for i in 0..n {
        if let Some(clue) = state.top[i] {
            let indices: Vec<usize> = (0..n).map(|r| r * n + i).collect();
            match analyze_line(
                state,
                n_val,
                clue,
                Line::Col(i),
                CluePosition::Top(i),
                &indices,
            ) {
                TechniqueResult::NoProgress => {}
                other => return other,
            }
        }
        if let Some(clue) = state.bottom[i] {
            let indices: Vec<usize> = (0..n).rev().map(|r| r * n + i).collect();
            match analyze_line(
                state,
                n_val,
                clue,
                Line::Col(i),
                CluePosition::Bottom(i),
                &indices,
            ) {
                TechniqueResult::NoProgress => {}
                other => return other,
            }
        }
        if let Some(clue) = state.left[i] {
            let indices: Vec<usize> = (0..n).map(|c| i * n + c).collect();
            match analyze_line(
                state,
                n_val,
                clue,
                Line::Row(i),
                CluePosition::Left(i),
                &indices,
            ) {
                TechniqueResult::NoProgress => {}
                other => return other,
            }
        }
        if let Some(clue) = state.right[i] {
            let indices: Vec<usize> = (0..n).rev().map(|c| i * n + c).collect();
            match analyze_line(
                state,
                n_val,
                clue,
                Line::Row(i),
                CluePosition::Right(i),
                &indices,
            ) {
                TechniqueResult::NoProgress => {}
                other => return other,
            }
        }
    }

    TechniqueResult::NoProgress
}

fn analyze_line(
    state: &mut SolveState,
    n_val: u8,
    clue: u8,
    line: Line,
    clue_pos: CluePosition,
    indices: &[usize],
) -> TechniqueResult {
    // Locate `n` in viewing order. If `n` isn't placed in this line, we
    // don't apply here.
    let Some(k) = indices
        .iter()
        .position(|&idx| state.grid[idx] == Some(n_val))
    else {
        return TechniqueResult::NoProgress;
    };

    // `clue - 1` is the number of visibles the k cells before n must
    // supply. Only the exact-saturation case (`clue - 1 == k`) gives us
    // an analytical conclusion — anything else is deferred.
    if (clue as usize) != k + 1 || k == 0 {
        return TechniqueResult::NoProgress;
    }

    let mut running_max: u8 = 0;
    let mut actions = Vec::new();

    for &idx in &indices[..k] {
        if let Some(v) = state.grid[idx] {
            // Already placed; it's a running-max by saturation, so update.
            // (The saturation conclusion itself guarantees v > running_max.)
            running_max = running_max.max(v);
            continue;
        }
        // Candidate must be > running_max and < n. Collect eliminations.
        let cands = state.candidates[idx];
        let r = idx / state.n;
        let c = idx % state.n;
        for v in cands.iter() {
            if v <= running_max || v >= n_val {
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: v,
                });
            }
        }
        // The minimum feasible value at this position is running_max + 1.
        // Use that as the new lower bound for the next position.
        running_max += 1;
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
        technique: Technique::VisibilityAnalysis,
        actions,
        reason: Reason::VisibilityForcing {
            line,
            clue: clue_pos,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::logic::state::SolveState;

    #[test]
    fn saturation_forces_single_cell_between_placed() {
        // n=5, Top=4, Col 2 in viewing order = [2, 3, _, 5, _].
        // n=5 at pos 3 → k=3. clue-1=3==k → saturation. Pre-n cells must
        // be ascending. pos 0=2 (OK), pos 1=3 (OK), pos 2 must be >3 and
        // <5 → among Latin-peer-legal candidates, R2C2 = 4.
        let mut board = Board::new_empty(5);
        board.set(0, 2, Some(2));
        board.set(1, 2, Some(3));
        board.set(3, 2, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_top(2, Some(4));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        let idx = state.idx(2, 2);
        assert_eq!(state.candidates[idx].singleton(), Some(4));

        if let TechniqueResult::Progress(step) = result {
            assert_eq!(step.technique, Technique::VisibilityAnalysis);
            assert!(matches!(
                step.reason,
                Reason::VisibilityForcing {
                    clue: CluePosition::Top(2),
                    ..
                }
            ));
        }
    }

    #[test]
    fn pattern_c_bracket_single_gap() {
        // n=7, Left=3, Row 0 in viewing order = [5, _, 7, _, _, _, _].
        // n=7 at pos 2 → k=2. clue-1=2==k → saturation. pos 0=5 (OK,
        // running_max=5), pos 1 must be >5 and <7 → only 6.
        let mut board = Board::new_empty(7);
        board.set(0, 0, Some(5));
        board.set(0, 2, Some(7));
        let mut clues = Clues::new_all_none(7);
        clues.set_left(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        let idx = state.idx(0, 1);
        assert_eq!(state.candidates[idx].singleton(), Some(6));
    }

    #[test]
    fn no_progress_when_n_not_placed() {
        // Empty 5×5, Left=3 on row 0. n=5 is not placed → skip.
        let board = Board::new_empty(5);
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }

    #[test]
    fn no_progress_when_under_saturated() {
        // n=5, Left=2, Row 0 = [_, _, _, 5, _]. k=3, clue-1=1<3 → defer.
        let mut board = Board::new_empty(5);
        board.set(0, 3, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }

    #[test]
    fn saturation_chains_across_multiple_free_cells() {
        // n=5, Left=4, Row 0 = [_, _, _, 5, _]. k=3, clue-1=3==k → ascending.
        // pos 0 must be >=1, pos 1 >=2, pos 2 >=3 (and <5, and != Latin peers).
        // After forcing: pos 0 candidates ⊆ {1,2,3,4}, pos 1 ⊆ {2,3,4},
        // pos 2 ⊆ {3,4}.
        let mut board = Board::new_empty(5);
        board.set(0, 3, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(4));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));
        // pos 1 loses 1, pos 2 loses 1 and 2.
        assert!(!state.candidates[state.idx(0, 1)].contains(1));
        assert!(!state.candidates[state.idx(0, 2)].contains(1));
        assert!(!state.candidates[state.idx(0, 2)].contains(2));
    }
}
