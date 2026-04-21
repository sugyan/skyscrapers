use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, CluePosition, Reason, Step, Technique};
use crate::logic::state::SolveState;

/// Apply clue-based pruning to narrow candidates.
///
/// Called once during initialization, before the solve loop. Returns `None`
/// if the pruning produces a contradiction (any cell with an empty candidate
/// set). Otherwise returns one [`Step`] per clue that actually changed
/// candidates, so the solve trace can begin by showing what the clues alone
/// forced before the solve loop even started.
pub(crate) fn apply(state: &mut SolveState) -> Option<Vec<Step>> {
    let n = state.n;
    let mut steps = Vec::new();

    for i in 0..n {
        if let Some(clue) = state.top[i] {
            let indices: Vec<usize> = (0..n).map(|r| r * n + i).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Top(i), &indices) {
                steps.push(step);
            }
        }
        if let Some(clue) = state.bottom[i] {
            let indices: Vec<usize> = (0..n).rev().map(|r| r * n + i).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Bottom(i), &indices) {
                steps.push(step);
            }
        }
        if let Some(clue) = state.left[i] {
            let indices: Vec<usize> = (0..n).map(|c| i * n + c).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Left(i), &indices) {
                steps.push(step);
            }
        }
        if let Some(clue) = state.right[i] {
            let indices: Vec<usize> = (0..n).rev().map(|c| i * n + c).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Right(i), &indices) {
                steps.push(step);
            }
        }
    }

    if !state.candidates.iter().all(|c| !c.is_empty()) {
        return None;
    }

    Some(steps)
}

/// Prune one line in viewing order and emit a single [`Step`] covering all
/// eliminations produced by this clue. Returns `None` if the clue changed
/// nothing (the Step would be empty).
fn apply_clue_to_line(
    state: &mut SolveState,
    clue: u8,
    clue_pos: CluePosition,
    indices: &[usize],
) -> Option<Step> {
    let n = state.n;
    let n_val = n as u8;
    let mut actions = Vec::new();

    if clue == 1 {
        // Only the tallest is visible → first cell in viewing order is n.
        let idx = indices[0];
        let before = state.candidates[idx];
        let after = before.intersect(Candidates::single(n_val));
        state.candidates[idx] = after;
        record_diff(n, idx, before, after, &mut actions);
    } else if clue == n_val {
        // All buildings visible → strictly ascending in viewing order.
        for (pos, &idx) in indices.iter().enumerate() {
            let forced = (pos as u8) + 1;
            let before = state.candidates[idx];
            let after = before.intersect(Candidates::single(forced));
            state.candidates[idx] = after;
            record_diff(n, idx, before, after, &mut actions);
        }
    } else {
        // Position `pos` (0-indexed from viewer) can have value at most
        // `n + 1 - clue + pos`, capped at n.
        for (pos, &idx) in indices.iter().enumerate() {
            let max_val = ((n + 1 - clue as usize + pos).min(n)) as u8;
            let before = state.candidates[idx];
            let mut after = before;
            for v in (max_val + 1)..=n_val {
                after = after.remove(v);
            }
            state.candidates[idx] = after;
            record_diff(n, idx, before, after, &mut actions);
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(Step {
            technique: Technique::CluePruning,
            actions,
            reason: Reason::InitialClueConstraint { clue: clue_pos },
        })
    }
}

/// Record an `Eliminate` action for each value removed going from `before`
/// to `after`.
fn record_diff(
    n: usize,
    idx: usize,
    before: Candidates,
    after: Candidates,
    actions: &mut Vec<Action>,
) {
    let r = idx / n;
    let c = idx % n;
    for v in 1..=(n as u8) {
        if before.contains(v) && !after.contains(v) {
            actions.push(Action::Eliminate {
                row: r,
                col: c,
                value: v,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn clue_1_forces_n_at_edge() {
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(1)); // Left clue = 1 on row 0 → (0,0) must be 4
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let (state, _) = SolveState::new(&puzzle).unwrap();
        assert_eq!(state.grid[0], Some(4));
    }

    #[test]
    fn clue_n_forces_ascending() {
        let mut clues = Clues::new_all_none(4);
        clues.set_top(0, Some(4));
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let (state, _) = SolveState::new(&puzzle).unwrap();
        for r in 0..4 {
            assert_eq!(state.grid[r * 4], Some(r as u8 + 1));
        }
    }

    #[test]
    fn clue_prunes_candidates() {
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(2));
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };
        let (state, _) = SolveState::new(&puzzle).unwrap();
        assert!(!state.candidates[0].contains(5));
        assert!(state.candidates[0].contains(4));
        assert!(state.candidates[1].contains(5));
    }

    #[test]
    fn contradictory_clues_detected() {
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(4));
        clues.set_right(0, Some(4));
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        assert!(SolveState::new(&puzzle).is_none());
    }

    #[test]
    fn apply_emits_step_per_pruning_clue() {
        // Left=5 on row 0 with n=5 forces ascending: every cell loses 4
        // candidates → one Step with 20 Eliminate actions.
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(5));
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };
        let (_, init_steps) = SolveState::new(&puzzle).unwrap();
        assert_eq!(init_steps.len(), 1);
        let step = &init_steps[0];
        assert_eq!(step.technique, Technique::CluePruning);
        match step.reason {
            Reason::InitialClueConstraint {
                clue: CluePosition::Left(0),
            } => {}
            ref other => panic!("unexpected reason: {other:?}"),
        }
        assert_eq!(step.actions.len(), 20);
    }

    #[test]
    fn apply_skips_trivial_clues() {
        // Top=3 on col 2 with n=5: only pos 0 loses {4,5} and pos 1 loses {5}.
        let mut clues = Clues::new_all_none(5);
        clues.set_top(2, Some(3));
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };
        let (_, init_steps) = SolveState::new(&puzzle).unwrap();
        assert_eq!(init_steps.len(), 1);
        assert_eq!(init_steps[0].actions.len(), 3);
    }
}
