use crate::logic::difficulty::{Action, Reason, Step, Technique};
use crate::logic::state::{SolveState, sees};
use crate::logic::techniques::TechniqueResult;

/// Bound on chain length. Skyscrapers is on n ≤ 9, so the bivalue graph stays
/// tiny, but the cap protects against pathological backtracking on large n.
const MAX_CHAIN_LENGTH: usize = 12;

/// XY-Chain: chain of bivalue cells `A₁, A₂, …, Aₖ` whose endpoints both
/// contain value `x` and whose adjacent cells share a row or column with
/// candidates relaying through a chain of values.
///
/// Concretely the search alternates "incoming" and "outgoing" values: at
/// step `i` the cell `Aᵢ = {pᵢ, qᵢ}` shares one of its values with its
/// neighbour `Aᵢ₋₁` (the incoming link) and the other with `Aᵢ₊₁` (the
/// outgoing link). Both endpoints carry `x` (the start as one of its two
/// values, the end as its outgoing value), but `x` may also appear at an
/// interior cell — that does not break the deduction, because the relay
/// keeps propagating: if `A₁ ≠ x` then `A₂` is forced, …, and ultimately
/// `Aₖ = x`. So in every case `x` must lie at `A₁` or `Aₖ`, and any cell
/// that sees *both* endpoints can therefore not be `x`.
///
/// Length 2 chains (a "hidden pair" inside a single line) are handled by
/// `NakedSets`; we require `length ≥ 3`. Length 3 corresponds to the classic
/// XY-Wing.
pub(crate) fn apply(state: &mut SolveState) -> TechniqueResult {
    let bivalues = collect_bivalue_cells(state);

    for start_idx in 0..bivalues.len() {
        let (start_cell, [v1, v2]) = bivalues[start_idx];
        for &(x, first_link) in &[(v1, v2), (v2, v1)] {
            let mut chain = vec![start_cell];
            if let Some(result) = dfs(state, &bivalues, first_link, x, &mut chain) {
                return result;
            }
        }
    }

    TechniqueResult::NoProgress
}

/// Collect all unsolved cells whose candidate set has exactly two values.
fn collect_bivalue_cells(state: &SolveState) -> Vec<((usize, usize), [u8; 2])> {
    let n = state.n;
    let mut out = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let idx = state.idx(r, c);
            if state.grid[idx].is_some() {
                continue;
            }
            if state.candidates[idx].count() != 2 {
                continue;
            }
            let mut it = state.candidates[idx].iter();
            let a = it.next().unwrap();
            let b = it.next().unwrap();
            out.push(((r, c), [a, b]));
        }
    }
    out
}

/// Extend the chain from its current tail, looking for a bivalue cell that
/// contains `expected_link` and shares a row/col with the tail. On reaching
/// an endpoint with outgoing value `x` (and chain length ≥ 3), try to
/// eliminate `x` from cells that see both endpoints.
fn dfs(
    state: &mut SolveState,
    bivalues: &[((usize, usize), [u8; 2])],
    expected_link: u8,
    x: u8,
    chain: &mut Vec<(usize, usize)>,
) -> Option<TechniqueResult> {
    if chain.len() >= MAX_CHAIN_LENGTH {
        return None;
    }
    let tail = *chain.last().unwrap();

    for &(cell, [a, b]) in bivalues {
        if chain.contains(&cell) {
            continue;
        }
        if !sees(tail, cell) {
            continue;
        }
        let next_outgoing = if a == expected_link {
            b
        } else if b == expected_link {
            a
        } else {
            continue;
        };

        chain.push(cell);

        if chain.len() >= 3 && next_outgoing == x {
            if let Some(result) = try_eliminate(state, chain, x) {
                return Some(result);
            }
        }

        if let Some(result) = dfs(state, bivalues, next_outgoing, x, chain) {
            return Some(result);
        }

        chain.pop();
    }

    None
}

/// Eliminate `x` from cells that see both endpoints of `chain` (and aren't
/// part of the chain themselves). Returns `None` if no elimination applies.
fn try_eliminate(
    state: &mut SolveState,
    chain: &[(usize, usize)],
    x: u8,
) -> Option<TechniqueResult> {
    let n = state.n;
    let first = chain[0];
    let last = *chain.last().unwrap();

    let mut actions = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let idx = state.idx(r, c);
            if state.grid[idx].is_some() || !state.candidates[idx].contains(x) {
                continue;
            }
            let cell = (r, c);
            if chain.contains(&cell) {
                continue;
            }
            if sees(cell, first) && sees(cell, last) {
                actions.push(Action::Eliminate {
                    row: r,
                    col: c,
                    value: x,
                });
            }
        }
    }

    if actions.is_empty() {
        return None;
    }

    for action in &actions {
        if let Action::Eliminate { row, col, value } = action {
            if !state.eliminate(*row, *col, *value) {
                return Some(TechniqueResult::Contradiction);
            }
        }
    }

    Some(TechniqueResult::Progress(Step {
        technique: Technique::XyChain,
        actions,
        reason: Reason::XyChainElimination {
            chain: chain.to_vec(),
            eliminated_value: x,
        },
    }))
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::candidates::Candidates;

    #[test]
    fn no_progress_on_empty_board() {
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();
        assert!(matches!(apply(&mut state), TechniqueResult::NoProgress));
    }

    #[test]
    fn finds_chain_of_length_4() {
        // Reproduce the n=4 seed=20260506 hard puzzle's mid-solve state, but on
        // a 5x5 board with manually planted candidates so we can exercise the
        // chain in isolation:
        //
        //   R1C1 {2,3} —col1— R3C1 {1,2} —row3— R3C3 {1,2} —col3— R4C3 {2,3}
        //
        //   Endpoint x = 3. Cell R4C1 sees R1C1 (col 1) and R4C3 (row 4),
        //   so 3 must be eliminated from R4C1.

        let board = Board::new_empty(5);
        let clues = Clues::new_all_none(5);
        let puzzle = Puzzle { board, clues };
        let mut state = SolveState::new(&puzzle).unwrap();

        let pair = |a: u8, b: u8| Candidates::single(a).union(Candidates::single(b));
        let i00 = state.idx(0, 0);
        let i20 = state.idx(2, 0);
        let i22 = state.idx(2, 2);
        let i32 = state.idx(3, 2);
        let i30 = state.idx(3, 0);
        state.candidates[i00] = pair(2, 3);
        state.candidates[i20] = pair(1, 2);
        state.candidates[i22] = pair(1, 2);
        state.candidates[i32] = pair(2, 3);

        assert!(state.candidates[i30].contains(3));

        let result = apply(&mut state);
        match result {
            TechniqueResult::Progress(step) => {
                assert_eq!(step.technique, Technique::XyChain);
                let has_target = step.actions.iter().any(|a| {
                    matches!(
                        a,
                        Action::Eliminate {
                            row: 3,
                            col: 0,
                            value: 3,
                        }
                    )
                });
                assert!(has_target, "Expected -3 from R4C1, got: {:?}", step.actions);
            }
            _ => panic!("Expected XY-Chain to find a pattern"),
        }
    }
}
