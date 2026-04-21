use crate::candidates::Candidates;
use crate::logic::difficulty::{Action, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Find hidden pairs/triples in rows and columns.
///
/// If k values (k=2,3) in a line can only appear in exactly k cells,
/// those cells can only contain those k values (eliminate other candidates).
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Check rows
    for r in 0..n {
        let indices: Vec<usize> = (0..n).map(|c| r * n + c).collect();
        let result = find_hidden_set(state, &indices, Line::Row(r));
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }

    // Check columns
    for c in 0..n {
        let indices: Vec<usize> = (0..n).map(|r| r * n + c).collect();
        let result = find_hidden_set(state, &indices, Line::Col(c));
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }

    TechniqueResult::NoProgress
}

fn find_hidden_set(state: &mut SolveState, indices: &[usize], line: Line) -> TechniqueResult {
    let n = state.n;

    // For each unplaced value, find which unassigned cells can hold it
    let mut value_positions: Vec<(u8, Vec<usize>)> = Vec::new();
    for v in 1..=n as u8 {
        // Skip values already placed in this line
        if indices.iter().any(|&idx| state.grid[idx] == Some(v)) {
            continue;
        }
        let positions: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&idx| state.grid[idx].is_none() && state.candidates[idx].contains(v))
            .collect();
        if !positions.is_empty() {
            value_positions.push((v, positions));
        }
    }

    // Try hidden pairs (k=2)
    for i in 0..value_positions.len() {
        for j in (i + 1)..value_positions.len() {
            let (v1, pos1) = &value_positions[i];
            let (v2, pos2) = &value_positions[j];
            // Union of positions where v1 or v2 can go
            let mut combined: Vec<usize> = pos1.clone();
            for &p in pos2 {
                if !combined.contains(&p) {
                    combined.push(p);
                }
            }
            if combined.len() == 2 {
                let hidden_values = Candidates::single(*v1).union(Candidates::single(*v2));
                let result =
                    restrict_to_hidden_set(state, &combined, hidden_values, &[*v1, *v2], line, n);
                if !matches!(result, TechniqueResult::NoProgress) {
                    return result;
                }
            }
        }
    }

    // Try hidden triples (k=3)
    for i in 0..value_positions.len() {
        for j in (i + 1)..value_positions.len() {
            for k in (j + 1)..value_positions.len() {
                let (v1, pos1) = &value_positions[i];
                let (v2, pos2) = &value_positions[j];
                let (v3, pos3) = &value_positions[k];
                let mut combined: Vec<usize> = pos1.clone();
                for &p in pos2 {
                    if !combined.contains(&p) {
                        combined.push(p);
                    }
                }
                for &p in pos3 {
                    if !combined.contains(&p) {
                        combined.push(p);
                    }
                }
                if combined.len() == 3 {
                    let hidden_values = Candidates::single(*v1)
                        .union(Candidates::single(*v2))
                        .union(Candidates::single(*v3));
                    let result = restrict_to_hidden_set(
                        state,
                        &combined,
                        hidden_values,
                        &[*v1, *v2, *v3],
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

/// Restrict the cells in `set_indices` to only contain `hidden_values`.
/// Eliminate any other candidates from these cells.
fn restrict_to_hidden_set(
    state: &mut SolveState,
    set_indices: &[usize],
    hidden_values: Candidates,
    values_list: &[u8],
    line: Line,
    n: usize,
) -> TechniqueResult {
    let mut actions = Vec::new();

    for &idx in set_indices {
        for v in state.candidates[idx].iter() {
            if !hidden_values.contains(v) {
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

    TechniqueResult::Progress(Step {
        technique: Technique::HiddenSets,
        actions,
        reason: Reason::SetInLine {
            line,
            cells,
            values: values_list.to_vec(),
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn no_contradiction_on_partially_filled_row() {
        // Smoke test: hidden-sets processing handles a partially filled row
        // without reporting a contradiction. After placing 1,2,3 in a 5×5 row,
        // propagation already resolves remaining cells to {4,5}, so no hidden
        // pair is needed.
        let mut board = Board::new_empty(5);
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(2));
        board.set(0, 2, Some(3));
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(!matches!(result, TechniqueResult::Contradiction));
    }

    #[test]
    fn no_hidden_set_in_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
