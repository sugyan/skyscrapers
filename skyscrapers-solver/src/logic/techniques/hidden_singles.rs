use crate::logic::difficulty::{Action, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Find and assign a hidden single (value that can only go in one cell within a line).
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Check rows
    for r in 0..n {
        for v in 1..=n as u8 {
            // Skip if already placed in this row
            if (0..n).any(|c| state.grid[state.idx(r, c)] == Some(v)) {
                continue;
            }
            let positions: Vec<usize> = (0..n)
                .filter(|&c| {
                    state.grid[state.idx(r, c)].is_none()
                        && state.candidates[state.idx(r, c)].contains(v)
                })
                .collect();
            if positions.is_empty() {
                return TechniqueResult::Contradiction;
            }
            if positions.len() == 1 {
                let c = positions[0];
                if !state.assign(r, c, v) {
                    return TechniqueResult::Contradiction;
                }
                return TechniqueResult::Progress(Step {
                    technique: Technique::HiddenSingles,
                    actions: vec![Action::Place {
                        row: r,
                        col: c,
                        value: v,
                    }],
                    reason: Reason::UniqueInLine {
                        line: Line::Row(r),
                        value: v,
                    },
                });
            }
        }
    }

    // Check columns
    for c in 0..n {
        for v in 1..=n as u8 {
            if (0..n).any(|r| state.grid[state.idx(r, c)] == Some(v)) {
                continue;
            }
            let positions: Vec<usize> = (0..n)
                .filter(|&r| {
                    state.grid[state.idx(r, c)].is_none()
                        && state.candidates[state.idx(r, c)].contains(v)
                })
                .collect();
            if positions.is_empty() {
                return TechniqueResult::Contradiction;
            }
            if positions.len() == 1 {
                let r = positions[0];
                if !state.assign(r, c, v) {
                    return TechniqueResult::Contradiction;
                }
                return TechniqueResult::Progress(Step {
                    technique: Technique::HiddenSingles,
                    actions: vec![Action::Place {
                        row: r,
                        col: c,
                        value: v,
                    }],
                    reason: Reason::UniqueInLine {
                        line: Line::Col(c),
                        value: v,
                    },
                });
            }
        }
    }

    TechniqueResult::NoProgress
}
