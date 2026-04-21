use crate::logic::difficulty::{Action, Line, Reason, Step, Technique};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Find X-Wing and Swordfish patterns.
///
/// X-Wing (k=2): If a value v appears in exactly 2 positions in each of 2 rows,
/// and those positions are in the same 2 columns, then v can be eliminated from
/// those columns in all other rows. Same logic applies with rows/columns swapped.
///
/// Swordfish (k=3): Same pattern extended to 3 rows and 3 columns.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    for v in 1..=n as u8 {
        // X-Wing / Swordfish on rows (eliminate from columns)
        let result = find_fish_rows(state, v, n);
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }

        // X-Wing / Swordfish on columns (eliminate from rows)
        let result = find_fish_cols(state, v, n);
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }

    TechniqueResult::NoProgress
}

/// Find X-Wing/Swordfish in rows for value v.
fn find_fish_rows(state: &mut SolveState, v: u8, n: usize) -> TechniqueResult {
    // For each row, find which columns have v as a candidate (in unassigned cells)
    let mut row_cols: Vec<(usize, Vec<usize>)> = Vec::new();
    for r in 0..n {
        // Skip if v is already placed in this row
        if (0..n).any(|c| state.grid[r * n + c] == Some(v)) {
            continue;
        }
        let cols: Vec<usize> = (0..n)
            .filter(|&c| {
                let idx = r * n + c;
                state.grid[idx].is_none() && state.candidates[idx].contains(v)
            })
            .collect();
        if cols.len() >= 2 && cols.len() <= 3 {
            row_cols.push((r, cols));
        }
    }

    // X-Wing (k=2)
    for i in 0..row_cols.len() {
        for j in (i + 1)..row_cols.len() {
            if row_cols[i].1.len() == 2
                && row_cols[j].1.len() == 2
                && row_cols[i].1 == row_cols[j].1
            {
                let rows = [row_cols[i].0, row_cols[j].0];
                let cols = &row_cols[i].1;
                let result = eliminate_fish(state, v, &rows, cols, true, n);
                if !matches!(result, TechniqueResult::NoProgress) {
                    return result;
                }
            }
        }
    }

    // Swordfish (k=3)
    for i in 0..row_cols.len() {
        for j in (i + 1)..row_cols.len() {
            for k in (j + 1)..row_cols.len() {
                let mut col_union: Vec<usize> = row_cols[i].1.clone();
                for &c in &row_cols[j].1 {
                    if !col_union.contains(&c) {
                        col_union.push(c);
                    }
                }
                for &c in &row_cols[k].1 {
                    if !col_union.contains(&c) {
                        col_union.push(c);
                    }
                }
                if col_union.len() == 3 {
                    let rows = [row_cols[i].0, row_cols[j].0, row_cols[k].0];
                    col_union.sort_unstable();
                    let result = eliminate_fish(state, v, &rows, &col_union, true, n);
                    if !matches!(result, TechniqueResult::NoProgress) {
                        return result;
                    }
                }
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Find X-Wing/Swordfish in columns for value v.
fn find_fish_cols(state: &mut SolveState, v: u8, n: usize) -> TechniqueResult {
    let mut col_rows: Vec<(usize, Vec<usize>)> = Vec::new();
    for c in 0..n {
        if (0..n).any(|r| state.grid[r * n + c] == Some(v)) {
            continue;
        }
        let rows: Vec<usize> = (0..n)
            .filter(|&r| {
                let idx = r * n + c;
                state.grid[idx].is_none() && state.candidates[idx].contains(v)
            })
            .collect();
        if rows.len() >= 2 && rows.len() <= 3 {
            col_rows.push((c, rows));
        }
    }

    // X-Wing (k=2)
    for i in 0..col_rows.len() {
        for j in (i + 1)..col_rows.len() {
            if col_rows[i].1.len() == 2
                && col_rows[j].1.len() == 2
                && col_rows[i].1 == col_rows[j].1
            {
                let cols = [col_rows[i].0, col_rows[j].0];
                let rows = &col_rows[i].1;
                // base_lines = cols (base columns), cover_lines = rows (cover rows)
                let result = eliminate_fish(state, v, &cols, rows, false, n);
                if !matches!(result, TechniqueResult::NoProgress) {
                    return result;
                }
            }
        }
    }

    // Swordfish (k=3)
    for i in 0..col_rows.len() {
        for j in (i + 1)..col_rows.len() {
            for k in (j + 1)..col_rows.len() {
                let mut row_union: Vec<usize> = col_rows[i].1.clone();
                for &r in &col_rows[j].1 {
                    if !row_union.contains(&r) {
                        row_union.push(r);
                    }
                }
                for &r in &col_rows[k].1 {
                    if !row_union.contains(&r) {
                        row_union.push(r);
                    }
                }
                if row_union.len() == 3 {
                    let cols = [col_rows[i].0, col_rows[j].0, col_rows[k].0];
                    row_union.sort_unstable();
                    // base_lines = cols (base columns), cover_lines = row_union (cover rows)
                    let result = eliminate_fish(state, v, &cols, &row_union, false, n);
                    if !matches!(result, TechniqueResult::NoProgress) {
                        return result;
                    }
                }
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Eliminate value v from cover lines, except at intersection cells.
///
/// If `base_is_rows` is true: base lines are rows, cover lines are columns.
/// Eliminate v from the cover columns in all rows NOT in the base set.
///
/// If `base_is_rows` is false: base lines are columns, cover lines are rows.
/// Eliminate v from the cover rows in all columns NOT in the base set.
fn eliminate_fish(
    state: &mut SolveState,
    v: u8,
    base_lines: &[usize],
    cover_lines: &[usize],
    base_is_rows: bool,
    n: usize,
) -> TechniqueResult {
    let mut actions = Vec::new();

    if base_is_rows {
        // Eliminate v from cover columns in non-base rows
        for &c in cover_lines {
            for r in 0..n {
                if base_lines.contains(&r) {
                    continue;
                }
                let idx = r * n + c;
                if state.grid[idx].is_none() && state.candidates[idx].contains(v) {
                    actions.push(Action::Eliminate {
                        row: r,
                        col: c,
                        value: v,
                    });
                }
            }
        }
    } else {
        // Eliminate v from cover rows in non-base columns
        for &r in cover_lines {
            for c in 0..n {
                if base_lines.contains(&c) {
                    continue;
                }
                let idx = r * n + c;
                if state.grid[idx].is_none() && state.candidates[idx].contains(v) {
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
        return TechniqueResult::NoProgress;
    }

    for action in &actions {
        if let Action::Eliminate { row, col, value } = action {
            if !state.eliminate(*row, *col, *value) {
                return TechniqueResult::Contradiction;
            }
        }
    }

    let (base, cover) = if base_is_rows {
        (
            base_lines.iter().map(|&r| Line::Row(r)).collect(),
            cover_lines.iter().map(|&c| Line::Col(c)).collect(),
        )
    } else {
        (
            base_lines.iter().map(|&c| Line::Col(c)).collect(),
            cover_lines.iter().map(|&r| Line::Row(r)).collect(),
        )
    };

    TechniqueResult::Progress(Step {
        technique: Technique::XWing,
        actions,
        reason: Reason::FishPattern {
            value: v,
            base_lines: base,
            cover_lines: cover,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    #[test]
    fn finds_x_wing_in_rows() {
        // 5×5 board. Value 3 appears in exactly cols {1, 3} in rows 0 and 2.
        // Should eliminate 3 from cols 1 and 3 in rows 1, 3, 4.
        let board = Board::new_empty(5);
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Set up: in rows 0 and 2, restrict value 3 to cols 1 and 3 only
        for c in 0..5 {
            if c != 1 && c != 3 {
                state.candidates[c] = state.candidates[c].remove(3);
                state.candidates[2 * 5 + c] = state.candidates[2 * 5 + c].remove(3);
            }
        }

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::XWing);
                // Should eliminate 3 from (1,1), (1,3), (3,1), (3,3), (4,1), (4,3)
                for action in &step.actions {
                    if let Action::Eliminate { row, col, value } = action {
                        assert_eq!(*value, 3);
                        assert!(*col == 1 || *col == 3);
                        assert!(*row != 0 && *row != 2);
                    }
                }
            }
            _ => panic!("Expected X-Wing to be found"),
        }
    }

    #[test]
    fn finds_x_wing_in_cols() {
        // 5×5 board. Value 2 appears in exactly rows {0, 3} in cols 1 and 4.
        // Should eliminate 2 from rows 0 and 3 in cols other than 1 and 4.
        let board = Board::new_empty(5);
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // In cols 1 and 4, restrict value 2 to rows 0 and 3 only
        for r in 0..5 {
            if r != 0 && r != 3 {
                state.candidates[r * 5 + 1] = state.candidates[r * 5 + 1].remove(2);
                state.candidates[r * 5 + 4] = state.candidates[r * 5 + 4].remove(2);
            }
        }

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::XWing);
                // Should eliminate 2 from rows 0 and 3 in cols other than 1 and 4
                for action in &step.actions {
                    if let Action::Eliminate { row, col, value } = action {
                        assert_eq!(*value, 2);
                        assert!(*row == 0 || *row == 3);
                        assert!(*col != 1 && *col != 4);
                    }
                }
            }
            _ => panic!("Expected column-based X-Wing to be found"),
        }
    }

    #[test]
    fn finds_swordfish_in_rows() {
        // 6×6 board. Value 1 appears in at most 3 columns across 3 rows,
        // and the union of those columns has exactly 3 elements.
        // Row 0: cols {0, 2}
        // Row 2: cols {0, 4}
        // Row 4: cols {2, 4}
        // Union = {0, 2, 4} — Swordfish pattern.
        // Should eliminate 1 from cols 0, 2, 4 in rows 1, 3, 5.
        let board = Board::new_empty(6);
        let clues = Clues::new_all_none(6);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        // Row 0: keep 1 only in cols 0 and 2
        for c in 0..6 {
            if c != 0 && c != 2 {
                state.candidates[c] = state.candidates[c].remove(1);
            }
        }
        // Row 2: keep 1 only in cols 0 and 4
        for c in 0..6 {
            if c != 0 && c != 4 {
                state.candidates[2 * 6 + c] = state.candidates[2 * 6 + c].remove(1);
            }
        }
        // Row 4: keep 1 only in cols 2 and 4
        for c in 0..6 {
            if c != 2 && c != 4 {
                state.candidates[4 * 6 + c] = state.candidates[4 * 6 + c].remove(1);
            }
        }

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::XWing);
                for action in &step.actions {
                    if let Action::Eliminate { row, col, value } = action {
                        assert_eq!(*value, 1);
                        assert!(*col == 0 || *col == 2 || *col == 4);
                        assert!(*row != 0 && *row != 2 && *row != 4);
                    }
                }
                assert!(!step.actions.is_empty());
            }
            _ => panic!("Expected Swordfish to be found"),
        }
    }

    #[test]
    fn no_x_wing_in_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        // In a completely empty 4x4, every value appears in all 4 columns per row,
        // so no X-Wing pattern exists
        assert!(matches!(result, TechniqueResult::NoProgress));
    }
}
