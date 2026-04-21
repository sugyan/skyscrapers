use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::{SolveState, sees};
use crate::logic::techniques::TechniqueResult;

/// A strong link: two cells in the same row or column where some value appears
/// as a candidate in exactly those two cells.
type StrongLink = ((usize, usize), (usize, usize));

/// W-Wing: two bivalue cells with the same candidate pair, connected by a strong link.
///
/// Cells A = {x, y} and B = {x, y}. A strong link on value v exists in some line
/// (v appears as candidate in exactly 2 cells C and D in that line), where C sees A
/// and D sees B (or vice versa).
///
/// Logic (using v = y as the linking value):
/// - If A = y → C can't be y → D must be y (strong link) → B can't be y → B = x
/// - If A = x → A = x
/// - Either way, at least one of A, B is x.
/// - Therefore, eliminate x from any cell that sees both A and B.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let n = state.n;

    // Collect bivalue cells: (row, col, v1, v2)
    let mut bivalues: Vec<(usize, usize, u8, u8)> = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if state.grid[idx].is_none() && state.candidates[idx].count() == 2 {
                let vals: Vec<u8> = state.candidates[idx].iter().collect();
                bivalues.push((r, c, vals[0], vals[1]));
            }
        }
    }

    // Precompute strong links: for each value v, lines where v appears in exactly 2 cells.
    let mut strong_links: Vec<Vec<StrongLink>> = vec![Vec::new(); n + 1];
    for v in 1..=n as u8 {
        // Check rows
        for r in 0..n {
            let cells_with_v: Vec<usize> = (0..n)
                .filter(|&c| {
                    let idx = r * n + c;
                    state.grid[idx].is_none() && state.candidates[idx].contains(v)
                })
                .collect();
            if cells_with_v.len() == 2 {
                strong_links[v as usize].push(((r, cells_with_v[0]), (r, cells_with_v[1])));
            }
        }
        // Check columns
        for c in 0..n {
            let cells_with_v: Vec<usize> = (0..n)
                .filter(|&r| {
                    let idx = r * n + c;
                    state.grid[idx].is_none() && state.candidates[idx].contains(v)
                })
                .collect();
            if cells_with_v.len() == 2 {
                strong_links[v as usize].push(((cells_with_v[0], c), (cells_with_v[1], c)));
            }
        }
    }

    // Try each pair of bivalue cells with the same candidate pair
    for (i, &(ar, ac, ax, ay)) in bivalues.iter().enumerate() {
        for &(br, bc, bx, by) in bivalues.iter().skip(i + 1) {
            // Must have the same candidate pair
            if !(ax == bx && ay == by) {
                continue;
            }

            // Try each value as the linking value
            for &link_val in &[ax, ay] {
                let elim_val = if link_val == ax { ay } else { ax };

                for &(c_cell, d_cell) in &strong_links[link_val as usize] {
                    // C and D must not be A or B themselves
                    if c_cell == (ar, ac)
                        || c_cell == (br, bc)
                        || d_cell == (ar, ac)
                        || d_cell == (br, bc)
                    {
                        continue;
                    }

                    // Check: C sees A and D sees B, or C sees B and D sees A
                    let c_sees_a = sees(c_cell, (ar, ac));
                    let d_sees_b = sees(d_cell, (br, bc));
                    let c_sees_b = sees(c_cell, (br, bc));
                    let d_sees_a = sees(d_cell, (ar, ac));

                    if !((c_sees_a && d_sees_b) || (c_sees_b && d_sees_a)) {
                        continue;
                    }

                    // Found W-Wing! Eliminate elim_val from cells seeing both A and B
                    let result = eliminate_w_wing(
                        state,
                        (ar, ac),
                        (br, bc),
                        c_cell,
                        d_cell,
                        link_val,
                        elim_val,
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

/// Eliminate `elim_val` from all cells that see both A and B.
fn eliminate_w_wing(
    state: &mut SolveState,
    cell_a: (usize, usize),
    cell_b: (usize, usize),
    link_c: (usize, usize),
    link_d: (usize, usize),
    link_val: u8,
    elim_val: u8,
) -> TechniqueResult {
    let n = state.n;
    let mut actions = Vec::new();

    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if state.grid[idx].is_some() || !state.candidates[idx].contains(elim_val) {
                continue;
            }
            if (r, c) == cell_a || (r, c) == cell_b {
                continue;
            }
            // Must see both A and B (share row or col with each)
            if sees((r, c), cell_a) && sees((r, c), cell_b) {
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: elim_val,
                });
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

    TechniqueResult::Progress(Step {
        technique: Technique::WWing,
        actions,
        reason: Reason::WWingElimination {
            cell_a,
            cell_b,
            link_c,
            link_d,
            link_value: link_val,
            eliminated_value: elim_val,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::candidates::Candidates;
    use crate::logic::state::SolveState;

    #[test]
    fn no_progress_without_bivalue_cells() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn finds_w_wing() {
        // 4x4 board with W-Wing pattern:
        // A (0,0) = {1,2}, B (1,3) = {1,2}
        // Strong link on 2 in row 2: (2,0) and (2,3) are the only cells with candidate 2
        // (2,0) sees A via col 0, (2,3) sees B via col 3
        // Logic: A=2 → (2,0)≠2 → (2,3)=2 → B≠2 → B=1. So at least one of A,B is 1.
        // Eliminate 1 from cells seeing both A and B.
        // (0,3) sees A via row 0 and B via col 3 → eliminate 1 from (0,3)
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        // Set up bivalue cells
        state.candidates[0] = Candidates::single(1).union(Candidates::single(2)); // (0,0)
        state.candidates[7] = Candidates::single(1).union(Candidates::single(2)); // (1,3)

        // Set up strong link on value 2 in row 2
        // (2,0) and (2,3) are the only cells with candidate 2 in row 2
        let n = state.n;
        for c in 0..n {
            let idx = 2 * n + c;
            state.candidates[idx] = state.candidates[idx].remove(2);
        }
        state.candidates[2 * n] = Candidates::single(2).union(Candidates::single(3)); // (2,0) = {2,3}
        state.candidates[2 * n + 3] = Candidates::single(2).union(Candidates::single(4)); // (2,3) = {2,4}

        // (0,3) should have candidate 1 before W-Wing
        assert!(state.candidates[3].contains(1));

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        // (0,3) should no longer have candidate 1
        assert!(!state.candidates[3].contains(1));
    }
}
