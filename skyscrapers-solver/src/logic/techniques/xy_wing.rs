use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::{SolveState, sees};
use crate::logic::techniques::TechniqueResult;

/// XY-Wing: three bivalue cells forming a pattern that eliminates a candidate.
///
/// Pivot A = {x, y}, Wing B = {x, z} (sees A), Wing C = {y, z} (sees A).
/// Any cell that sees both B and C can have z eliminated, because:
/// - If A = x, then B must be z
/// - If A = y, then C must be z
/// - Either way, z is in B or C, so any cell seeing both cannot be z.
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

    // Try each bivalue cell as pivot
    for &(pr, pc, x, y) in &bivalues {
        // Find wing B: sees pivot, has {x, z} where z != y
        for &(br, bc, bv1, bv2) in &bivalues {
            if (br, bc) == (pr, pc) {
                continue;
            }
            // B must see A (same row or column)
            if !sees((br, bc), (pr, pc)) {
                continue;
            }
            // B must share exactly one value with A, and the shared value must be x or y
            let (shared, z) = if bv1 == x && bv2 != y {
                (x, bv2)
            } else if bv2 == x && bv1 != y {
                (x, bv1)
            } else if bv1 == y && bv2 != x {
                (y, bv2)
            } else if bv2 == y && bv1 != x {
                (y, bv1)
            } else {
                continue;
            };

            // The other value of pivot (not shared with B)
            let other = if shared == x { y } else { x };

            // Find wing C: sees pivot, has {other, z}, on different line than B-pivot connection
            for &(cr, cc, cv1, cv2) in &bivalues {
                if (cr, cc) == (pr, pc) || (cr, cc) == (br, bc) {
                    continue;
                }
                // C must see A (same row or column)
                if !sees((cr, cc), (pr, pc)) {
                    continue;
                }
                // C must be on a different line from the B-pivot connection
                // If B sees pivot via row, C must see pivot via column (and vice versa)
                if br == pr && cr == pr {
                    continue; // both on same row as pivot
                }
                if bc == pc && cc == pc {
                    continue; // both on same column as pivot
                }
                // C must have {other, z}
                if !((cv1 == other && cv2 == z) || (cv1 == z && cv2 == other)) {
                    continue;
                }

                // Found XY-Wing pattern! Eliminate z from cells that see both B and C
                let result = eliminate_xy_wing(state, (pr, pc), (br, bc), (cr, cc), z);
                if !matches!(result, TechniqueResult::NoProgress) {
                    return result;
                }
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Eliminate z from all cells that see both wing_a and wing_b.
fn eliminate_xy_wing(
    state: &mut SolveState,
    pivot: (usize, usize),
    wing_a: (usize, usize),
    wing_b: (usize, usize),
    z: u8,
) -> TechniqueResult {
    let n = state.n;
    let mut actions = Vec::new();

    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if state.grid[idx].is_some() || !state.candidates[idx].contains(z) {
                continue;
            }
            if (r, c) == pivot || (r, c) == wing_a || (r, c) == wing_b {
                continue;
            }
            // Must see both wing_a and wing_b (share row or col with each)
            if sees((r, c), wing_a) && sees((r, c), wing_b) {
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: z,
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
        technique: Technique::XYWing,
        actions,
        reason: Reason::XYWingElimination {
            pivot,
            wing_a,
            wing_b,
            eliminated_value: z,
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
        // All cells have 4 candidates, no bivalue cells
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn finds_xy_wing() {
        // Set up a 4x4 board with an XY-Wing pattern:
        // Pivot (0,0) = {1,2}, Wing A (0,1) = {1,3}, Wing C (1,0) = {2,3}
        // Cell (1,1) sees both wings → eliminate 3
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        // Manually set up candidates
        let n = state.n;
        // Clear all candidates first
        for idx in 0..n * n {
            state.candidates[idx] = Candidates::all(n as u8);
        }

        // Set bivalue cells
        state.candidates[0] = Candidates::single(1).union(Candidates::single(2)); // (0,0) = {1,2}
        state.candidates[1] = Candidates::single(1).union(Candidates::single(3)); // (0,1) = {1,3}
        state.candidates[n] = Candidates::single(2).union(Candidates::single(3)); // (1,0) = {2,3}

        // (1,1) should have 3 as candidate so it can be eliminated
        // It sees wing A (0,1) via col 1, and wing C (1,0) via row 1
        assert!(state.candidates[n + 1].contains(3));

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        // (1,1) should no longer have candidate 3
        assert!(!state.candidates[n + 1].contains(3));
    }
}
