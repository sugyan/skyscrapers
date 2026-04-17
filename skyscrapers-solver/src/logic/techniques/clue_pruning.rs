use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, CluePosition, Reason, Step, Technique};
use crate::logic::state::SolveState;

/// Apply clue-based pruning to narrow candidates.
///
/// Called once during initialization, before the solve loop. Returns `None`
/// if a contradiction is detected (empty candidate set after pruning).
/// Otherwise returns one [`Step`] per clue that actually changed candidates,
/// so the trace can show why initial placements / eliminations are visible
/// from the first line of output.
pub(crate) fn apply(state: &mut SolveState) -> Option<Vec<Step>> {
    let n = state.n;
    let mut steps = Vec::new();

    for i in 0..n {
        // Top clues (looking down column i)
        if let Some(clue) = state.top[i] {
            let indices: Vec<usize> = (0..n).map(|r| r * n + i).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Top(i), &indices) {
                steps.push(step);
            }
        }

        // Bottom clues (looking up column i)
        if let Some(clue) = state.bottom[i] {
            let indices: Vec<usize> = (0..n).rev().map(|r| r * n + i).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Bottom(i), &indices) {
                steps.push(step);
            }
        }

        // Left clues (looking right along row i)
        if let Some(clue) = state.left[i] {
            let indices: Vec<usize> = (0..n).map(|c| i * n + c).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Left(i), &indices) {
                steps.push(step);
            }
        }

        // Right clues (looking left along row i)
        if let Some(clue) = state.right[i] {
            let indices: Vec<usize> = (0..n).rev().map(|c| i * n + c).collect();
            if let Some(step) = apply_clue_to_line(state, clue, CluePosition::Right(i), &indices) {
                steps.push(step);
            }
        }
    }

    // Check for empty candidates (contradiction)
    if !state.candidates.iter().all(|c| !c.is_empty()) {
        return None;
    }

    Some(steps)
}

/// Apply a single clue to one line (viewing order) and collect eliminations
/// as a [`Step`]. Returns `None` if the clue changed nothing.
fn apply_clue_to_line(
    state: &mut SolveState,
    clue: u8,
    clue_pos: CluePosition,
    indices: &[usize],
) -> Option<Step> {
    let n = indices.len() as u8;
    let mut actions = Vec::new();

    if clue == 1 {
        // Only the tallest building is visible → first cell (in viewing order) must be n
        let idx = indices[0];
        let before = state.candidates[idx];
        let after = before.intersect(Candidates::single(n));
        state.candidates[idx] = after;
        record_eliminations(state.n, idx, before, after, &mut actions);
    } else if clue == n {
        // All buildings visible → strict ascending in viewing order
        for (pos, &idx) in indices.iter().enumerate() {
            let forced = (pos as u8) + 1;
            let before = state.candidates[idx];
            let after = before.intersect(Candidates::single(forced));
            state.candidates[idx] = after;
            record_eliminations(state.n, idx, before, after, &mut actions);
        }
    } else {
        // Position `pos` (0-indexed from viewer) can have value at most
        // `n + 1 - clue + pos` (capped at n).
        for (pos, &idx) in indices.iter().enumerate() {
            let max_val = ((n as usize + 1 - clue as usize + pos).min(n as usize)) as u8;
            let before = state.candidates[idx];
            for v in (max_val + 1)..=n {
                if before.contains(v) {
                    state.candidates[idx] = state.candidates[idx].remove(v);
                    let r = idx / state.n;
                    let c = idx % state.n;
                    actions.push(Action::Eliminate {
                        row: r,
                        col: c,
                        value: v,
                    });
                }
            }
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

/// Record `Eliminate` actions for every value removed going from `before` to
/// `after`.
fn record_eliminations(
    n: usize,
    idx: usize,
    before: Candidates,
    after: Candidates,
    actions: &mut Vec<Action>,
) {
    let r = idx / n;
    let c = idx % n;
    for v in 1..=9u8 {
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
        // (0,0) should be determined as 4
        assert_eq!(state.grid[0], Some(4));
    }

    #[test]
    fn clue_n_forces_ascending() {
        let mut clues = Clues::new_all_none(4);
        clues.set_top(0, Some(4)); // Top clue = n on col 0 → ascending: 1,2,3,4
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
        clues.set_left(0, Some(2)); // Left clue = 2 on row 0
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };
        let (state, _) = SolveState::new(&puzzle).unwrap();
        // Position 0: max value = 5+1-2+0 = 4, so 5 should be removed
        assert!(!state.candidates[0].contains(5));
        assert!(state.candidates[0].contains(4));
        // Position 1: max value = 5+1-2+1 = 5, no restriction
        assert!(state.candidates[1].contains(5));
    }

    #[test]
    fn contradictory_clues_detected() {
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(4)); // row 0 ascending: 1,2,3,4
        clues.set_right(0, Some(4)); // row 0 descending: 4,3,2,1
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        assert!(SolveState::new(&puzzle).is_none());
    }

    #[test]
    fn apply_returns_steps_for_each_clue() {
        // Left=5 on row 0 with n=5 forces ascending, producing one Step
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(5));
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };

        let mut state = super::super::super::state::SolveState {
            n: 5,
            grid: vec![None; 25],
            candidates: vec![Candidates::all(5); 25],
            top: vec![None; 5],
            bottom: vec![None; 5],
            left: (0..5).map(|i| puzzle.clues.left(i)).collect(),
            right: vec![None; 5],
        };

        let steps = apply(&mut state).expect("no contradiction");
        assert_eq!(steps.len(), 1);
        let step = &steps[0];
        assert_eq!(step.technique, Technique::CluePruning);
        match step.reason {
            Reason::InitialClueConstraint {
                clue: CluePosition::Left(0),
            } => {}
            _ => panic!("unexpected reason: {:?}", step.reason),
        }
        // Five cells, each loses 4 candidates → 20 Eliminate actions
        assert_eq!(step.actions.len(), 20);
    }

    #[test]
    fn apply_skips_trivial_clues() {
        // Clue = 3 with n = 5: position 0 caps at max_val = 3 (removes {4, 5})
        let mut clues = Clues::new_all_none(5);
        clues.set_top(2, Some(3));
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };

        let mut state = super::super::super::state::SolveState {
            n: 5,
            grid: vec![None; 25],
            candidates: vec![Candidates::all(5); 25],
            top: (0..5).map(|i| puzzle.clues.top(i)).collect(),
            bottom: vec![None; 5],
            left: vec![None; 5],
            right: vec![None; 5],
        };

        let steps = apply(&mut state).expect("no contradiction");
        assert_eq!(steps.len(), 1);
        let step = &steps[0];
        // Position 0 (R0C2): remove {4,5}. Position 1 (R1C2): remove {5}.
        // Positions 2..=4 unchanged. Total = 3 eliminations.
        assert_eq!(step.actions.len(), 3);
    }
}
