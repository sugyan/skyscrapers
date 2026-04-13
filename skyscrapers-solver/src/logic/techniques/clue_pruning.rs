use crate::candidates::Candidates;
use crate::logic::state::SolveState;

/// Apply clue-based pruning to narrow candidates.
///
/// Called once during initialization, before the solve loop.
/// Returns false if a contradiction is detected (empty candidate set).
pub(crate) fn apply(state: &mut SolveState) -> bool {
    let n = state.n as u8;

    for i in 0..state.n {
        // Top clues (looking down column i)
        if let Some(clue) = state.top[i] {
            if clue == 1 {
                // Only the tallest building is visible → first cell must be n
                state.candidates[i] = state.candidates[i].intersect(Candidates::single(n));
            } else if clue == n {
                // All buildings visible → ascending order
                for r in 0..state.n {
                    state.candidates[r * state.n + i] = state.candidates[r * state.n + i]
                        .intersect(Candidates::single(r as u8 + 1));
                }
            } else {
                // Position r can have at most (n + 1 - clue + r) as value
                for r in 0..state.n {
                    let max_val = (n as usize + 1 - clue as usize + r).min(n as usize) as u8;
                    for v in (max_val + 1)..=n {
                        state.candidates[r * state.n + i] =
                            state.candidates[r * state.n + i].remove(v);
                    }
                }
            }
        }

        // Bottom clues (looking up column i)
        if let Some(clue) = state.bottom[i] {
            if clue == 1 {
                let idx = (state.n - 1) * state.n + i;
                state.candidates[idx] = state.candidates[idx].intersect(Candidates::single(n));
            } else if clue == n {
                for r in 0..state.n {
                    let idx = (state.n - 1 - r) * state.n + i;
                    state.candidates[idx] =
                        state.candidates[idx].intersect(Candidates::single(r as u8 + 1));
                }
            } else {
                for r in 0..state.n {
                    let dist = state.n - 1 - r;
                    let max_val = (n as usize + 1 - clue as usize + dist).min(n as usize) as u8;
                    for v in (max_val + 1)..=n {
                        state.candidates[r * state.n + i] =
                            state.candidates[r * state.n + i].remove(v);
                    }
                }
            }
        }

        // Left clues (looking right along row i)
        if let Some(clue) = state.left[i] {
            if clue == 1 {
                let idx = i * state.n;
                state.candidates[idx] = state.candidates[idx].intersect(Candidates::single(n));
            } else if clue == n {
                for c in 0..state.n {
                    let idx = i * state.n + c;
                    state.candidates[idx] =
                        state.candidates[idx].intersect(Candidates::single(c as u8 + 1));
                }
            } else {
                for c in 0..state.n {
                    let max_val = (n as usize + 1 - clue as usize + c).min(n as usize) as u8;
                    for v in (max_val + 1)..=n {
                        state.candidates[i * state.n + c] =
                            state.candidates[i * state.n + c].remove(v);
                    }
                }
            }
        }

        // Right clues (looking left along row i)
        if let Some(clue) = state.right[i] {
            if clue == 1 {
                let idx = i * state.n + state.n - 1;
                state.candidates[idx] = state.candidates[idx].intersect(Candidates::single(n));
            } else if clue == n {
                for c in 0..state.n {
                    let idx = i * state.n + (state.n - 1 - c);
                    state.candidates[idx] =
                        state.candidates[idx].intersect(Candidates::single(c as u8 + 1));
                }
            } else {
                for c in 0..state.n {
                    let dist = state.n - 1 - c;
                    let max_val = (n as usize + 1 - clue as usize + dist).min(n as usize) as u8;
                    for v in (max_val + 1)..=n {
                        state.candidates[i * state.n + c] =
                            state.candidates[i * state.n + c].remove(v);
                    }
                }
            }
        }
    }

    // Check for empty candidates
    state.candidates.iter().all(|c| !c.is_empty())
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;

    #[test]
    fn clue_1_forces_n_at_edge() {
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(1)); // Left clue = 1 on row 0 → (0,0) must be 4
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let state = SolveState::new(&puzzle).unwrap();
        // (0,0) should be determined as 4
        assert_eq!(state.grid[0], Some(4));
    }

    #[test]
    fn clue_n_forces_ascending() {
        let mut clues = Clues::new_all_none(4);
        clues.set_top(0, Some(4)); // Top clue = n on col 0 → ascending: 1,2,3,4
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let state = SolveState::new(&puzzle).unwrap();
        for r in 0..4 {
            assert_eq!(state.grid[r * 4], Some(r as u8 + 1));
        }
    }

    #[test]
    fn clue_prunes_candidates() {
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(2)); // Left clue = 2 on row 0
        let board = Board::new_empty(5);
        let puzzle = Puzzle { board, clues };
        let state = SolveState::new(&puzzle).unwrap();
        // Position 0: max value = 5+1-2+0 = 4, so 5 should be removed
        assert!(!state.candidates[0].contains(5));
        assert!(state.candidates[0].contains(4));
        // Position 1: max value = 5+1-2+1 = 5, no restriction
        assert!(state.candidates[1].contains(5));
    }

    #[test]
    fn contradictory_clues_detected() {
        let mut clues = Clues::new_all_none(4);
        // Contradictory: left=1 means (0,0)=4, but top=1 means (0,0)=4 — consistent
        // Let's make a real contradiction: left=4 means ascending, right=4 means descending
        clues.set_left(0, Some(4)); // row 0 ascending: 1,2,3,4
        clues.set_right(0, Some(4)); // row 0 descending: 4,3,2,1
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        // This should detect contradiction (ascending AND descending impossible for n>1)
        assert!(SolveState::new(&puzzle).is_none());
    }
}
