// Only compiled when the `analysis-hooks` feature is enabled, so production
// builds (CLI / wasm) neither see this analysis-specific surface in the
// public API nor pay any cost for it. Call sites in `techniques::` are
// `cfg`-gated to match.
#[cfg(feature = "analysis-hooks")]
pub mod analysis_hooks;
pub mod difficulty;
mod state;
mod techniques;

use skyscrapers_core::{Puzzle, Solution};

use crate::Solver;
use difficulty::{Action, Difficulty, Step, Technique};
use state::SolveState;
use techniques::{TechniqueResult, apply_next_technique};

/// Result of solving with the logic solver.
pub struct SolveResult {
    pub solutions: Vec<Solution>,
    pub difficulty: Option<Difficulty>,
    pub steps: Vec<Step>,
}

/// A solver that uses human-like logical techniques.
///
/// Applies techniques in order of difficulty. Does not use backtracking;
/// if no technique can make progress, the puzzle is reported as unsolvable.
pub struct LogicSolver;

impl LogicSolver {
    /// Solve the puzzle and return solutions, difficulty, and steps.
    ///
    /// The logic solver never branches, so it returns either 0 or 1 solution.
    /// `limit == 0` is the only value that short-circuits to an empty result;
    /// any positive `limit` runs the full solve. The parameter exists for
    /// [`Solver`] trait compatibility with [`BacktrackingSolver`](crate::BacktrackingSolver).
    pub fn solve_with_difficulty(&self, puzzle: &Puzzle, limit: usize) -> SolveResult {
        if limit == 0 {
            return SolveResult {
                solutions: Vec::new(),
                difficulty: None,
                steps: Vec::new(),
            };
        }

        let Some(mut state) = SolveState::new(puzzle) else {
            return SolveResult {
                solutions: Vec::new(),
                difficulty: None,
                steps: Vec::new(),
            };
        };
        let init_steps = std::mem::take(&mut state.init_steps);

        // Seed the trace with init-time CluePruning Steps so downstream
        // NakedSingles placements have a visible antecedent, but do NOT
        // promote `max_technique` from them — init pruning runs
        // unconditionally from clue geometry and its work is implicit in
        // the puzzle's starting state. Difficulty stays driven purely by
        // the techniques the solve loop actually needs from here.
        let mut steps = init_steps;
        let mut max_technique: Option<Technique> = None;

        loop {
            // Check if already complete (initial board values + propagation may solve it)
            if state.is_complete() {
                if state.verify_clues() {
                    return SolveResult {
                        solutions: vec![state.to_solution()],
                        difficulty: Some(
                            max_technique.map_or(Difficulty::Easy, |t| t.difficulty()),
                        ),
                        steps,
                    };
                } else {
                    // Grid is complete but violates clues — contradiction
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
            }

            match apply_next_technique(&mut state) {
                TechniqueResult::Progress(step) => {
                    let technique = step.technique;
                    steps.push(step);
                    match max_technique {
                        Some(current) if technique > current => {
                            max_technique = Some(technique);
                        }
                        None => {
                            max_technique = Some(technique);
                        }
                        _ => {}
                    }
                }
                TechniqueResult::Contradiction => {
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
                TechniqueResult::NoProgress => {
                    // Stuck — cannot solve with logic alone
                    return SolveResult {
                        solutions: Vec::new(),
                        difficulty: None,
                        steps,
                    };
                }
            }
        }
    }

    /// Get the next logical step from the current board state (for hints).
    pub fn next_step(&self, puzzle: &Puzzle, board: &skyscrapers_core::Board) -> Option<Step> {
        self.next_step_with_candidates(puzzle, board, None)
            .map(|(step, _)| step)
    }

    /// Like [`Self::next_step`], but also returns the solver's candidate
    /// snapshot at the moment the step was produced.
    ///
    /// The candidate grid is `n × n`; each cell lists the values the solver
    /// considers possible there. Hint UIs use this to compare against the
    /// user's pencil marks and surface missing/extra candidates.
    ///
    /// `user_candidates` is an optional `n × n` grid of the values the user
    /// has currently pencilled in. When supplied, steps whose every
    /// elimination action is already absorbed by the user's pencil marks
    /// (i.e. the eliminated value is absent from a non-empty user candidate
    /// set) are skipped — the solver still applies them internally so its
    /// state advances, but they are not surfaced as hints. This prevents
    /// the same hint from being returned over and over after the user has
    /// already synced or applied it. User candidates are used purely as a
    /// filter; they never feed back into the solver's deductions, so wrong
    /// pencil marks cannot poison the reasoning.
    pub fn next_step_with_candidates(
        &self,
        puzzle: &Puzzle,
        board: &skyscrapers_core::Board,
        user_candidates: Option<&[Vec<Vec<u8>>]>,
    ) -> Option<(Step, Vec<Vec<Vec<u8>>>)> {
        let hint_puzzle = Puzzle {
            board: board.clone(),
            clues: puzzle.clues.clone(),
        };

        let mut state = SolveState::new(&hint_puzzle)?;

        // Walk init-time CluePruning steps first, then drive the solve loop.
        // For each produced step, skip it if every action is already
        // absorbed by the user's pencil marks; the state has still been
        // mutated, so the next call will pick up where we left off. Move
        // init_steps out into a queue so absorbed entries can be popped
        // one at a time without dropping the rest.
        let mut init_steps = std::mem::take(&mut state.init_steps).into_iter();
        loop {
            if let Some(step) = init_steps.next() {
                if !is_step_absorbed(&step, board, user_candidates) {
                    let candidates = state.candidates_snapshot();
                    return Some((step, candidates));
                }
                continue;
            }

            match apply_next_technique(&mut state) {
                TechniqueResult::Progress(step) => {
                    if is_step_absorbed(&step, board, user_candidates) {
                        continue;
                    }
                    let candidates = state.candidates_snapshot();
                    return Some((step, candidates));
                }
                _ => return None,
            }
        }
    }
}

/// True iff every action in `step` is already reflected by the user's state.
///
/// A `Place` action is never considered absorbed here: if the user had
/// already entered the value, [`SolveState::new`] would have applied it via
/// `assign` and the technique loop wouldn't re-emit the placement. An
/// `Eliminate` action is absorbed when the cell is unconfirmed and the
/// value is missing from a *non-empty* user candidate set (an empty user
/// candidate set means "no pencil marks yet", which we cannot interpret as
/// an absorption).
fn is_step_absorbed(
    step: &Step,
    board: &skyscrapers_core::Board,
    user_candidates: Option<&[Vec<Vec<u8>>]>,
) -> bool {
    let Some(uc) = user_candidates else {
        return false;
    };
    if step.actions.is_empty() {
        return false;
    }
    step.actions.iter().all(|action| match *action {
        Action::Place { .. } => false,
        Action::Eliminate { row, col, value } => {
            if board.get(row, col).is_some() {
                return true;
            }
            let cell = &uc[row][col];
            !cell.is_empty() && !cell.contains(&value)
        }
    })
}

impl Solver for LogicSolver {
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution> {
        self.solve_with_difficulty(puzzle, limit).solutions
    }
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle, Solution};

    use super::*;

    #[test]
    fn solve_full_board() {
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let clues = Clues::from_solution(&sol);
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
    }

    #[test]
    fn solve_empty_returns_empty_when_stuck() {
        // Empty board with no clues — logic solver cannot solve
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert!(result.solutions.is_empty());
    }

    #[test]
    fn solve_near_complete_with_naked_singles() {
        // Board missing only a few cells, solvable by naked singles alone
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        // Fill all but last column
        for r in 0..4 {
            for c in 0..3 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
        assert_eq!(result.difficulty, Some(Difficulty::Easy));
    }

    #[test]
    fn solve_with_hidden_singles() {
        // Place values so that hidden singles are needed:
        // col 0 is fully determined, cols 1-3 have gaps requiring hidden singles
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        // Fill column 0 and some diagonal cells to create hidden single opportunities
        for r in 0..4 {
            board.set(r, 0, Some(sol.get(r, 0)));
        }
        // Add enough to force unique solution via hidden singles
        board.set(0, 1, Some(1));
        board.set(1, 1, Some(4));
        board.set(2, 2, Some(2));
        board.set(3, 3, Some(4));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        assert_eq!(result.solutions[0], sol);
    }

    #[test]
    fn solve_with_difficulty_reports_easy() {
        // A near-complete board solved entirely during init (propagation)
        // should report Easy difficulty
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..3 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1);
        // Solved during init (no explicit steps), but still Easy
        assert_eq!(result.difficulty, Some(Difficulty::Easy));
    }

    /// Helper to build a puzzle from clue specification.
    fn build_puzzle_with_clues(
        n: usize,
        givens: &[(usize, usize, u8)],
        top: &[(usize, u8)],
        bottom: &[(usize, u8)],
        left: &[(usize, u8)],
        right: &[(usize, u8)],
    ) -> Puzzle {
        let mut board = Board::new_empty(n);
        for &(r, c, v) in givens {
            board.set(r, c, Some(v));
        }
        let mut clues = Clues::new_all_none(n);
        for &(i, v) in top {
            clues.set_top(i, Some(v));
        }
        for &(i, v) in bottom {
            clues.set_bottom(i, Some(v));
        }
        for &(i, v) in left {
            clues.set_left(i, Some(v));
        }
        for &(i, v) in right {
            clues.set_right(i, Some(v));
        }
        Puzzle { board, clues }
    }

    #[test]
    fn solve_n4_seed9_with_forcing_chain() {
        // n=4, seed=9: 0 givens, clues: top=[2,_,2,2], left=[_,_,2,_], bottom=[_,_,2,_]
        // Requires FullForcingChain (Master) because SimpleForcingChain's basic propagation
        // (NakedSingles + HiddenSingles only) is insufficient to detect contradictions.
        let puzzle =
            build_puzzle_with_clues(4, &[], &[(0, 2), (2, 2), (3, 2)], &[(2, 2)], &[(2, 2)], &[]);
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=9 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Master));
    }

    #[test]
    fn solve_n4_seed13() {
        // n=4, seed=13: 2 givens, clues: top=[_,2,_,_], left=[_,_,1,_], bottom=[_,_,_,2]
        // givens: (2,1)=2, (3,0)=1
        // Hardest technique is single-clue permutation enumeration on small
        // (≤3-cell) lines, which is classified as `SimplePermutation` (Hard).
        let puzzle = build_puzzle_with_clues(
            4,
            &[(2, 1, 2), (3, 0, 1)],
            &[(1, 2)],
            &[(3, 2)],
            &[(2, 1)],
            &[],
        );
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=13 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Hard));
    }

    #[test]
    fn solve_n4_seed20260506_uses_xy_chain_at_hard() {
        // The puzzle generated for n=4 seed=20260506 difficulty=hard.
        // Clues: top=[_,4,1,_], left=[_,1,_,_], right=[_,_,_,3].
        // The mid-solve state has bivalue cells forming an XY-Chain that
        // eliminates a candidate; without XYChain, this puzzle requires
        // ALS-XZ (Expert) or ForcingChain (Master).
        let puzzle = build_puzzle_with_clues(4, &[], &[(1, 4), (2, 1)], &[], &[(1, 1)], &[(3, 3)]);
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "puzzle should solve");
        assert_eq!(result.difficulty, Some(Difficulty::Hard));
        assert!(
            result
                .steps
                .iter()
                .any(|s| s.technique == Technique::XyChain),
            "expected an XyChain step, got: {:?}",
            result.steps.iter().map(|s| s.technique).collect::<Vec<_>>()
        );
    }

    #[test]
    fn solve_n4_seed15() {
        // n=4, seed=15: 0 givens, clues: left=[_,2,_,3], right=[_,_,3,1], bottom=[3,_,_,_]
        // Hardest technique is single-clue permutation enumeration on small
        // (≤3-cell) lines, which is classified as `SimplePermutation` (Hard).
        let puzzle =
            build_puzzle_with_clues(4, &[], &[], &[(0, 3)], &[(1, 2), (3, 3)], &[(2, 3), (3, 1)]);
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.solutions.len(), 1, "n=4 seed=15 should be solvable");
        assert_eq!(result.difficulty, Some(Difficulty::Hard));
    }

    #[test]
    fn next_step_returns_hint() {
        // Board with a hidden single available
        let mut board = Board::new_empty(4);
        // Place values so that 4 in row 0 can only go at col 2 (hidden single)
        board.set(1, 0, Some(3));
        board.set(1, 1, Some(4));
        board.set(2, 0, Some(4));
        board.set(3, 0, Some(1));
        board.set(3, 3, Some(4));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle {
            board: board.clone(),
            clues,
        };

        let step = LogicSolver.next_step(&puzzle, &board);
        assert!(step.is_some());
        let step = step.unwrap();
        assert!(!step.actions.is_empty());
    }

    #[test]
    fn next_step_returns_init_clue_pruning_before_solve_loop() {
        // A pristine board with an actionable clue (Left=5 on n=5 saturates
        // Row 0). The hint should surface CluePruning first, not skip ahead
        // to a HiddenSingle derived from the hydrated state.
        let board = Board::new_empty(5);
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(5));
        let puzzle = Puzzle {
            board: board.clone(),
            clues,
        };

        let step = LogicSolver.next_step(&puzzle, &board).unwrap();
        assert_eq!(step.technique, Technique::CluePruning);
        assert!(matches!(
            step.reason,
            difficulty::Reason::InitialClueConstraint { .. }
        ));
    }

    #[test]
    fn absorbed_init_step_is_skipped() {
        // Same setup as the previous test: Left=5 on a 5×5 prunes Row 0
        // into the strictly ascending sequence 1..=5. Once the user has
        // pencilled in those exact candidates for Row 0, the CluePruning
        // step's eliminations are all "absorbed" — calling the hint API
        // again should look past this absorbed init step and either
        // surface a different step or return None when nothing's left.
        let board = Board::new_empty(5);
        let mut clues = Clues::new_all_none(5);
        clues.set_left(0, Some(5));
        let puzzle = Puzzle {
            board: board.clone(),
            clues,
        };

        // Mirror the post-CluePruning candidates the user would Sync into.
        // Row 0 is forced into the strictly ascending sequence 1..=5; peers
        // in each column lose the corresponding value via propagation.
        let user_candidates: Vec<Vec<Vec<u8>>> = (0..5)
            .map(|r| {
                (0..5)
                    .map(|c| {
                        let v = (c as u8) + 1;
                        if r == 0 {
                            vec![v]
                        } else {
                            (1..=5).filter(|&x| x != v).collect()
                        }
                    })
                    .collect()
            })
            .collect();

        let step = LogicSolver
            .next_step_with_candidates(&puzzle, &board, Some(&user_candidates))
            .map(|(s, _)| s);

        // The absorbed CluePruning step is gone. Whatever the next
        // emitted step is (if any), it must NOT be the same
        // InitialClueConstraint we just absorbed.
        if let Some(step) = step {
            assert_ne!(step.technique, Technique::CluePruning);
        }
    }
}
