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
