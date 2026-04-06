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

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn finds_hidden_single_in_row() {
        // Create a 4x4 board where value 4 can only go in one position in row 0
        // by filling column constraints
        let mut board = Board::new_empty(4);
        // Put 4 in col 0, 1, 2 (rows 1, 2, 3) so row 0 can only have 4 at col 3
        board.set(1, 0, Some(4));
        board.set(2, 1, Some(4));
        board.set(3, 2, Some(4));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Value 4 in row 0 can only be at col 3 (eliminated from col 0,1,2 by column peers)
        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::HiddenSingles);
                assert!(step.actions.contains(&Action::Place {
                    row: 0,
                    col: 3,
                    value: 4,
                }));
            }
            _ => panic!("Expected hidden single to find value 4 at (0,3)"),
        }
    }

    #[test]
    fn no_hidden_single_in_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
