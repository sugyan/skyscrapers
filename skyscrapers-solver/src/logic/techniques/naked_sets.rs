use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Find naked pairs/triples in rows and columns.
///
/// If k cells (k=2,3) in a line share exactly k candidate values,
/// those values can be eliminated from all other cells in the line.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Check rows
    for r in 0..n {
        let indices: Vec<usize> = (0..n).map(|c| r * n + c).collect();
        let result = find_naked_set(state, &indices, Line::Row(r));
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }

    // Check columns
    for c in 0..n {
        let indices: Vec<usize> = (0..n).map(|r| r * n + c).collect();
        let result = find_naked_set(state, &indices, Line::Col(c));
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }

    TechniqueResult::NoProgress
}

fn find_naked_set(state: &mut SolveState, indices: &[usize], line: Line) -> TechniqueResult {
    let n = state.n;
    // Collect unassigned cells in this line
    let unassigned: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|&idx| state.grid[idx].is_none())
        .collect();

    if unassigned.len() < 3 {
        return TechniqueResult::NoProgress; // too few cells for a useful naked set
    }

    // Try pairs (k=2)
    for i in 0..unassigned.len() {
        for j in (i + 1)..unassigned.len() {
            let union = state.candidates[unassigned[i]].union(state.candidates[unassigned[j]]);
            if union.count() == 2 {
                let result = eliminate_naked_set(
                    state,
                    &unassigned,
                    &[unassigned[i], unassigned[j]],
                    union,
                    line,
                    n,
                );
                if !matches!(result, TechniqueResult::NoProgress) {
                    return result;
                }
            }
        }
    }

    // Try triples (k=3)
    for i in 0..unassigned.len() {
        for j in (i + 1)..unassigned.len() {
            for k in (j + 1)..unassigned.len() {
                let union = state.candidates[unassigned[i]]
                    .union(state.candidates[unassigned[j]])
                    .union(state.candidates[unassigned[k]]);
                if union.count() == 3 {
                    let result = eliminate_naked_set(
                        state,
                        &unassigned,
                        &[unassigned[i], unassigned[j], unassigned[k]],
                        union,
                        line,
                        n,
                    );
                    if !matches!(result, TechniqueResult::NoProgress) {
                        return result;
                    }
                }
            }
        }
    }

    TechniqueResult::NoProgress
}

fn eliminate_naked_set(
    state: &mut SolveState,
    unassigned: &[usize],
    set_indices: &[usize],
    set_values: Candidates,
    line: Line,
    n: usize,
) -> TechniqueResult {
    let mut actions = Vec::new();

    // Collect eliminations first
    for &idx in unassigned {
        if set_indices.contains(&idx) {
            continue;
        }
        for v in set_values.iter() {
            if state.candidates[idx].contains(v) {
                let r = idx / n;
                let c = idx % n;
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: v,
                });
            }
        }
    }

    if actions.is_empty() {
        return TechniqueResult::NoProgress;
    }

    // Apply eliminations
    for action in &actions {
        if let Action::Eliminate { row, col, value } = action {
            if !state.eliminate(*row, *col, *value) {
                return TechniqueResult::Contradiction;
            }
        }
    }

    let cells: Vec<(usize, usize)> = set_indices.iter().map(|&idx| (idx / n, idx % n)).collect();
    let values: Vec<u8> = set_values.iter().collect();

    TechniqueResult::Progress(Step {
        technique: Technique::NakedSets,
        actions,
        reason: Reason::SetInLine {
            line,
            cells,
            values,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn finds_naked_pair_in_row() {
        // 5×5 board. In row 0: place 1 at col 0, 2 at col 1.
        // Then eliminate candidates so that cols 2 and 3 both have only {3,4}.
        // This is a naked pair: 3 and 4 should be eliminated from col 4.
        let mut board = Board::new_empty(5);
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(2));
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Manually restrict candidates for row 0:
        // col 2: {3, 4}, col 3: {3, 4}, col 4: {3, 4, 5}
        let idx2 = state.idx(0, 2);
        let idx3 = state.idx(0, 3);
        let idx4 = state.idx(0, 4);
        state.candidates[idx2] = Candidates::single(3).union(Candidates::single(4));
        state.candidates[idx3] = Candidates::single(3).union(Candidates::single(4));
        state.candidates[idx4] = Candidates::single(3)
            .union(Candidates::single(4))
            .union(Candidates::single(5));

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::NakedSets);
                // Should eliminate 3 and 4 from col 4
                assert!(step.actions.contains(&Action::Eliminate {
                    row: 0,
                    col: 4,
                    value: 3,
                }));
                assert!(step.actions.contains(&Action::Eliminate {
                    row: 0,
                    col: 4,
                    value: 4,
                }));
            }
            _ => panic!("Expected naked pair to be found"),
        }
    }

    #[test]
    fn no_naked_set_in_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
