use super::super::difficulty::{Action, Reason, Step, Technique};
use super::super::state::SolveState;
use super::{TechniqueResult, propagate, propagate_simple};

/// SimpleForcingChain: assume a candidate value, propagate with NakedSingles + HiddenSingles only.
/// Human-traceable assumption chains.
pub(crate) fn apply_simple(state: &mut SolveState) -> TechniqueResult {
    apply_inner(state, false)
}

/// FullForcingChain: assume a candidate value, propagate with all techniques.
/// Only tried after SimpleForcingChain fails.
pub(crate) fn apply_full(state: &mut SolveState) -> TechniqueResult {
    apply_inner(state, true)
}

/// Shared implementation for Simple and Full ForcingChain.
/// `full=false`: propagate with NakedSingles + HiddenSingles only (Simple).
/// `full=true`: propagate with all techniques except ForcingChain (Full).
fn apply_inner(state: &mut SolveState, full: bool) -> TechniqueResult {
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

    let technique = if full {
        Technique::FullForcingChain
    } else {
        Technique::SimpleForcingChain
    };

    for &(r, c, _) in &cells {
        let idx = r * n + c;
        let candidates: Vec<u8> = state.candidates[idx].iter().collect();

        for &v in &candidates {
            let mut trial = state.clone();

            // Try assigning this candidate
            let contradiction = if !trial.assign(r, c, v) {
                true
            } else if full {
                !propagate(&mut trial)
            } else {
                !propagate_simple(&mut trial)
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
                    technique,
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
        let (mut state, _) = SolveState::new(&puzzle).unwrap();
        // No clues, so forcing chain cannot find contradictions
        assert!(matches!(
            apply_simple(&mut state),
            TechniqueResult::NoProgress
        ));
        assert!(matches!(
            apply_full(&mut state),
            TechniqueResult::NoProgress
        ));
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
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        // Run simpler techniques first to get to a state where forcing chain is needed
        if !propagate(&mut state) {
            panic!("Unexpected contradiction during propagation");
        }
        // Check if forcing chain can now make progress (try full propagation)
        let found_forcing_chain = match apply_full(&mut state) {
            TechniqueResult::Progress(step) => {
                assert!(matches!(
                    step.technique,
                    Technique::SimpleForcingChain | Technique::FullForcingChain
                ));
                assert!(!step.actions.is_empty());
                true
            }
            TechniqueResult::NoProgress => {
                // Propagation didn't help and forcing chain found nothing.
                // The state may already be fully reduced by other techniques.
                false
            }
            TechniqueResult::Contradiction => {
                panic!("Unexpected contradiction from forcing chain");
            }
        };
        // Either forcing chain fired, or the puzzle was solved without it
        assert!(
            found_forcing_chain || state.is_complete(),
            "Expected forcing chain to make progress or puzzle to be solved"
        );
    }
}
