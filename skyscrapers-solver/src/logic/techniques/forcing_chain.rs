use super::super::difficulty::{Action, Reason, Step, Technique};
use super::super::state::SolveState;
use super::{TechniqueResult, propagate};

/// Forcing Chain: assume a candidate value, propagate, and eliminate if contradiction.
///
/// For each unassigned cell with 2-3 candidates, try each candidate:
/// 1. Clone the state and assign the candidate
/// 2. Run all other techniques to propagate
/// 3. If contradiction is detected, eliminate that candidate from the original state
///
/// Depth-1 only (no nested assumptions).
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Collect unassigned cells sorted by candidate count (ascending)
    let mut cells: Vec<(usize, usize, u32)> = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if state.grid[idx].is_none() {
                let count = state.candidates[idx].count();
                if (2..=3).contains(&count) {
                    cells.push((r, c, count));
                }
            }
        }
    }
    cells.sort_by_key(|&(_, _, count)| count);

    for &(r, c, _) in &cells {
        let idx = r * n + c;
        let candidates: Vec<u8> = state.candidates[idx].iter().collect();

        for &v in &candidates {
            let mut trial = state.clone();

            // Try assigning this candidate
            let contradiction = if !trial.assign(r, c, v) {
                true
            } else {
                // Propagate using all techniques except ForcingChain
                !propagate(&mut trial)
            };

            if contradiction {
                // This candidate leads to contradiction — eliminate it
                if !state.eliminate(r, c, v) {
                    return TechniqueResult::Contradiction;
                }

                let actions = vec![Action::Eliminate {
                    row: r,
                    col: c,
                    value: v,
                }];

                return TechniqueResult::Progress(Step {
                    technique: Technique::ForcingChain,
                    actions,
                    reason: Reason::ForcingChainElimination {
                        assumed_cell: (r, c),
                        assumed_value: v,
                    },
                });
            }
        }
    }

    TechniqueResult::NoProgress
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::super::super::difficulty::Technique;
    use super::super::super::state::SolveState;
    use super::super::TechniqueResult;
    use super::*;

    #[test]
    fn no_progress_on_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        // No clues, so forcing chain cannot find contradictions
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn eliminates_contradictory_candidate() {
        // n=4 puzzle where forcing chain can eliminate a candidate
        // seed=9 from the analysis: 0 givens, 5 clues
        //     2 . 2 2
        //   +---------+
        // . | . . . . | .
        // . | . . . . | .
        // 2 | . . . . | .
        // . | . . . . | .
        //   +---------+
        //     . . 2 .
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_top(0, Some(2));
        clues.set_top(2, Some(2));
        clues.set_top(3, Some(2));
        clues.set_left(2, Some(2));
        clues.set_bottom(2, Some(2));
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Run simpler techniques first to get to a state where forcing chain is needed
        loop {
            if !propagate(&mut state) {
                panic!("Unexpected contradiction during propagation");
            }
            // Check if forcing chain can now make progress
            let result = apply(&mut state);
            match result {
                TechniqueResult::Progress(step) => {
                    assert_eq!(step.technique, Technique::ForcingChain);
                    assert!(!step.actions.is_empty());
                    return; // success
                }
                TechniqueResult::NoProgress => {
                    // Propagation didn't help and forcing chain found nothing
                    // This puzzle may already be fully reduced
                    break;
                }
                TechniqueResult::Contradiction => {
                    panic!("Unexpected contradiction from forcing chain");
                }
            }
        }
        // If we get here, forcing chain wasn't triggered (unexpected for this puzzle)
        panic!("Expected forcing chain to find a contradiction");
    }

}
