use crate::logic::difficulty::{
    Action, CluePosition, Line, Reason, Step, Technique, VisibilityPattern,
};
use crate::logic::state::SolveState;
use crate::logic::techniques::TechniqueResult;

/// Analytical visibility forcing.
///
/// Unlike [`permutation`](super::permutation), which enumerates per-cell
/// assignments exhaustively, this technique reasons about a clued line
/// symbolically by combining three simple observations:
///
/// * **Dead zone (Pattern A):** free cells that lie *past* an already-placed
///   `n` in viewing order can never contribute visibility. They are
///   treated as non-viable when counting still-available visible slots.
/// * **Saturation (Pattern B):** if the number of remaining visibles needed
///   (`clue - currently-visible`) equals the number of viable free cells,
///   every one of those cells must become a new running maximum. Walking
///   the line in viewing order, each viable cell is then forced to exceed
///   the running max to date, with each subsequent viable requiring an
///   additional +1.
/// * **Bracket (Pattern C):** the degenerate case of saturation with a single
///   viable cell — e.g. `[5, ?, 7, ?, ?, ?, ?]` with Left=3 and n=7 forces
///   the single gap to satisfy `5 < v < 7`, pinning it to 6. Reported with
///   [`VisibilityPattern::Bracket`] so trace consumers can label it
///   distinctly.
///
/// Pattern A is used as input to B/C — it never eliminates on its own.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    for cl in state.clued_lines() {
        let result = analyze_line(state, &cl.indices, cl.expected, cl.line, cl.clue_pos);
        if !matches!(result, TechniqueResult::NoProgress) {
            return result;
        }
    }
    TechniqueResult::NoProgress
}

fn analyze_line(
    state: &mut SolveState,
    indices: &[usize],
    clue: u8,
    line: Line,
    clue_pos: CluePosition,
) -> TechniqueResult {
    let n = state.n;
    let n_val = n as u8;

    // Pattern A: locate placed `n` (if any) to mark dead-zone boundary.
    let n_pos: Option<usize> = indices
        .iter()
        .position(|&idx| state.grid[idx] == Some(n_val));

    let last_viable_excl = n_pos.map(|p| p + 1).unwrap_or(indices.len());

    // Scan once to compute current visibility (placed-only) and collect
    // viable free positions (free + before the dead-zone boundary).
    let mut current_visible: u8 = 0;
    let mut max_placed: u8 = 0;
    let mut viable: Vec<usize> = Vec::new();
    for (pos, &idx) in indices.iter().enumerate() {
        match state.grid[idx] {
            Some(v) => {
                if v > max_placed {
                    current_visible += 1;
                    max_placed = v;
                }
            }
            None => {
                if pos < last_viable_excl {
                    viable.push(pos);
                }
            }
        }
    }

    // `current_visible` counts placed-only visibles. It can legitimately
    // exceed `clue` mid-solve: future placements in earlier empty cells can
    // hide buildings currently counted as visible, so this is NOT a sound
    // contradiction signal here. Similarly, `needed > viable.len()` is not
    // sound because a larger placed value further along the line can retro-
    // actively cancel a currently-visible placement, shrinking `current_
    // visible` and thus raising `needed`. Leave these cases to techniques
    // that reason over full assignments (permutation / forcing chains).
    if current_visible > clue {
        return TechniqueResult::NoProgress;
    }
    let needed = clue - current_visible;

    if (needed as usize) > viable.len() {
        return TechniqueResult::NoProgress;
    }

    // Pattern B / C: saturation. Every viable free cell must become a new
    // running maximum. When a single viable cell is bracketed between
    // placed values we still take this path, but label it `Bracket` for
    // readability.
    if needed > 0 && (needed as usize) == viable.len() {
        let pattern = if viable.len() == 1 {
            VisibilityPattern::Bracket
        } else {
            VisibilityPattern::Saturation
        };
        return saturation_forcing(state, indices, &viable, pattern, line, clue_pos, n);
    }

    TechniqueResult::NoProgress
}

/// Apply saturation forcing: walk the line in viewing order, requiring
/// each viable free cell to strictly exceed the running max so far, and
/// grow the running max by at least +1 at each viable.
fn saturation_forcing(
    state: &mut SolveState,
    indices: &[usize],
    viable: &[usize],
    pattern: VisibilityPattern,
    line: Line,
    clue_pos: CluePosition,
    n: usize,
) -> TechniqueResult {
    let mut running_max: u8 = 0;
    let mut viable_idx = 0usize;
    let mut actions: Vec<Action> = Vec::new();

    for (pos, &idx) in indices.iter().enumerate() {
        if viable_idx < viable.len() && viable[viable_idx] == pos {
            viable_idx += 1;
            // Cell must be > running_max. Collect eliminations.
            let cands = state.candidates[idx];
            for v in cands.iter() {
                if v <= running_max {
                    actions.push(Action::Eliminate {
                        row: idx / n,
                        col: idx % n,
                        value: v,
                    });
                }
            }
            // Minimum possible new running max after this viable.
            running_max += 1;
        } else if let Some(v) = state.grid[idx] {
            if v > running_max {
                running_max = v;
            }
        }
        // Dead-zone free cells: skip — they don't update running_max.
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
        technique: Technique::VisibilityAnalysis,
        actions,
        reason: Reason::VisibilityForcing {
            line,
            clue: clue_pos,
            pattern,
        },
    })
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::logic::state::SolveState;

    #[test]
    fn bracket_pattern_single_free_between_placed() {
        // n=7, row 0 viewing order: [5, ?, 7, ?, ?, ?, ?], Left=3.
        // Placed 7 at pos 2 → dead zone = pos 3..6, viable = {pos 1}.
        // current_visible from placed (5, 7) = 2, needed = 1 = viable.len().
        // pos 0 placed 5 → running=5. pos 1 viable must be > 5 → only 6
        // survives (7 already used in row).
        let mut board = Board::new_empty(7);
        board.set(0, 0, Some(5));
        board.set(0, 2, Some(7));
        let mut clues = Clues::new_all_none(7);
        clues.set_left(0, Some(3));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        let idx = state.idx(0, 1);
        assert_eq!(state.candidates[idx].singleton(), Some(6));

        if let TechniqueResult::Progress(step) = result {
            assert_eq!(step.technique, Technique::VisibilityAnalysis);
            match step.reason {
                Reason::VisibilityForcing {
                    pattern: VisibilityPattern::Bracket,
                    ..
                } => {}
                other => panic!("expected Bracket pattern, got {:?}", other),
            }
        }
    }

    #[test]
    fn saturation_forces_single_cell_in_partial_col() {
        // n=5, col 2 viewing order (Top): [2, 3, ?, 5, ?], Top=4.
        // Placed 5 at pos 3 → dead zone = pos 4, viable = {pos 2}.
        // current_visible = 3 (2, 3, 5), needed = 1. pos 2 must be > 3,
        // and the peer row/column elimination leaves only {4}.
        let mut board = Board::new_empty(5);
        board.set(0, 2, Some(2));
        board.set(1, 2, Some(3));
        board.set(3, 2, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_top(2, Some(4));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        let idx = state.idx(2, 2);
        assert_eq!(state.candidates[idx].singleton(), Some(4));
    }

    #[test]
    fn contradiction_when_needed_exceeds_viable() {
        // n=4, row 0: [_, _, _, 4] placed, Left=3.
        // n is at pos 3 → dead zone = (nothing past pos 3), viable =
        // {pos 0, 1, 2}. current_visible from placed = 1 (just 4).
        // needed = 2 which is ≤ 3, so no contradiction from this clue alone.
        // To force a contradiction: Left=1 on a row where n is NOT at pos 0.
        let mut board = Board::new_empty(4);
        board.set(0, 3, Some(4));
        let mut clues = Clues::new_all_none(4);
        // Left=1 forces pos 0 = n, but n is already placed elsewhere.
        // clue_pruning catches this during init; SolveState::new returns None.
        clues.set_left(0, Some(1));
        let puzzle = Puzzle { board, clues };
        assert!(SolveState::new(&puzzle).is_none());
    }

    #[test]
    fn no_progress_without_saturation() {
        // Empty 4×4 board with a non-extreme clue — analytical reasoning
        // shouldn't fire until some placements narrow things down.
        let board = Board::new_empty(4);
        let mut clues = Clues::new_all_none(4);
        clues.set_left(0, Some(2));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::NoProgress));
    }

    #[test]
    fn multi_viable_saturation_chains_ascending() {
        // n=5, row 0: [_, _, _, 5, _], Left=4.
        // Placed 5 at pos 3 → dead zone = pos 4, viable = {0, 1, 2}.
        // current_visible = 1 (just 5), needed = 3 = viable.len().
        // All three viables must be ascending: pos 0 ≥ 1, pos 1 ≥ 2,
        // pos 2 ≥ 3. Combined with `!= 5` (Latin), candidates become
        // {1,2,3,4}, {2,3,4}, {3,4}.
        let mut board = Board::new_empty(5);
        board.set(0, 3, Some(5));
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(4));
        let puzzle = Puzzle { board, clues };
        let (mut state, _) = SolveState::new(&puzzle).unwrap();

        let result = apply(&mut state);
        assert!(matches!(result, TechniqueResult::Progress(_)));

        // pos 1 (row 0, col 1) must lose value 1.
        assert!(!state.candidates[state.idx(0, 1)].contains(1));
        // pos 2 (row 0, col 2) must lose values 1 and 2.
        assert!(!state.candidates[state.idx(0, 2)].contains(1));
        assert!(!state.candidates[state.idx(0, 2)].contains(2));
    }
}
