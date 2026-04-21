use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Find and assign a naked single (cell with exactly one candidate).
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    for r in 0..state.n {
        for c in 0..state.n {
            let idx = state.idx(r, c);
            if state.grid[idx].is_some() {
                continue;
            }
            if let Some(v) = state.candidates[idx].singleton() {
                if !state.assign(r, c, v) {
                    return TechniqueResult::Contradiction;
                }
                return TechniqueResult::Progress(Step {
                    technique: Technique::NakedSingles,
                    actions: vec![Action::Place {
                        row: r,
                        col: c,
                        value: v,
                    }],
                    reason: Reason::SingleCandidate { row: r, col: c },
                });
            }
        }
    }
    TechniqueResult::NoProgress
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn apply_resolves_forced_single_after_init() {
        // Place all but one value in a row. After init, (0,3) has only one
        // candidate (4) but is not yet assigned — eliminate() no longer
        // auto-assigns singletons. NakedSingles::apply picks it up.
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(2));
        board.set(0, 2, Some(3));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        // After init, (0,3) has single candidate 4 but is not placed yet.
        assert_eq!(state.grid[state.idx(0, 3)], None);
        assert_eq!(state.candidates[state.idx(0, 3)].singleton(), Some(4));

        // NakedSingles::apply places it as an explicit step.
        match apply(&mut state) {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.actions.len(), 1);
            }
            _ => panic!("expected Progress"),
        }
        assert_eq!(state.grid[state.idx(0, 3)], Some(4));
    }

    #[test]
    fn no_naked_single_when_multiple_candidates() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
