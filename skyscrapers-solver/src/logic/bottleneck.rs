//! Stage 2 difficulty-texture metric: **bottleneck count**.
//!
//! The logic solver's tier rating (Easy…Master) hides a wide within-tier
//! spread. The greedy single-path trace (Stage 1) counts many hard "stalls"
//! that a human collapses into a single insight, so it fails to rank puzzles
//! the way they feel. This metric instead counts *genuine bottlenecks*: how
//! many times the solver must reach for the top tier `D` before a subsequent
//! cascade of cheaper (`< D`) techniques finishes the surrounding work.
//!
//! Because every technique is a monotone, sound elimination, closing under any
//! subset of tiers reaches a unique order-independent fixpoint. So one
//! "bottleneck round" is well defined: at the current `< D` fixpoint, take
//! *all* available tier-`D` moves at once (their count is the round's
//! **width** — a forgiveness measure), apply them, then re-close under `< D`
//! (the number of cells placed by that closure is the round's **cascade** — how
//! much a single key unlocks). Repeat until solved; the number of rounds is the
//! bottleneck count.
//!
//! Dev-only, gated behind the `analysis-hooks` feature.

use skyscrapers_core::Puzzle;

use super::difficulty::Difficulty;
use super::state::SolveState;
use super::techniques::{TECHNIQUES, apply_step, find_all_for_tier, propagate_with};
use crate::LogicSolver;

/// One bottleneck round: a sweep of top-tier keys plus the cheap cascade it
/// unlocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BottleneckRound {
    /// Number of distinct top-tier steps available at the stall (forgiveness).
    pub width: usize,
    /// Cells placed by the subsequent `< D` closure (the key's cascade power).
    pub cascade: usize,
}

/// Result of the bottleneck-count analysis for a single puzzle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottleneckProfile {
    /// The hardest tier the logic solver needed, or `None` if it could not
    /// solve the puzzle (in which case `rounds` is empty and `solved` false).
    pub top_tier: Option<Difficulty>,
    /// One entry per bottleneck round, in solve order.
    pub rounds: Vec<BottleneckRound>,
    /// Whether the round loop drove the grid to completion.
    pub solved: bool,
}

impl BottleneckProfile {
    fn empty() -> Self {
        Self {
            top_tier: None,
            rounds: Vec::new(),
            solved: false,
        }
    }

    /// Number of genuine bottlenecks — the headline metric.
    ///
    /// One round = one wave of top-tier reasoning: at the current `< D` stall,
    /// apply *every* available tier-`D` move, then re-close under `< D`. The
    /// count is how many times the cheap techniques stall and the solver must
    /// inject hard reasoning to move on. Hand-solving (2026-07) confirmed this
    /// tracks felt difficulty within a tier: a long forced grind of
    /// candidate-only eliminations (many rounds) feels hard even when it ends
    /// in a single big cascade, so buildup rounds must be counted, not
    /// collapsed. `21` and `25` (both very hard, indistinguishable to the
    /// solver) score 5; the easy seeds score 1.
    pub fn bottlenecks(&self) -> usize {
        self.rounds.len()
    }

    /// Rounds whose keys unlocked ≥1 placement (`cascade > 0`) — the "release"
    /// count. Texture only: distinguishes staged puzzles (several releases)
    /// from grind-then-flow puzzles (one big release after buildup).
    pub fn releases(&self) -> usize {
        self.rounds.iter().filter(|r| r.cascade > 0).count()
    }

    /// Rounds that only shaved candidates (`cascade == 0`) — the forced grind
    /// before a release. Texture only: high buildup = "eliminate a lot before
    /// anything moves".
    pub fn buildup_rounds(&self) -> usize {
        self.rounds.iter().filter(|r| r.cascade == 0).count()
    }

    /// Smallest round width; `1` marks a truly forced round (a single key).
    pub fn min_width(&self) -> Option<usize> {
        self.rounds.iter().map(|r| r.width).min()
    }

    /// Total top-tier steps applied across all rounds (Σ width).
    pub fn total_top_steps(&self) -> usize {
        self.rounds.iter().map(|r| r.width).sum()
    }

    /// Largest single-round cascade.
    pub fn max_cascade(&self) -> usize {
        self.rounds.iter().map(|r| r.cascade).max().unwrap_or(0)
    }
}

fn placed(state: &SolveState) -> usize {
    state.grid.iter().filter(|c| c.is_some()).count()
}

/// Compute the bottleneck profile for `puzzle`.
///
/// See the module docs for the round semantics. Returns an empty profile when
/// the logic solver cannot solve the puzzle or the state is contradictory.
pub fn compute_bottlenecks(puzzle: &Puzzle) -> BottleneckProfile {
    // 1. Determine the top tier the logic solver needs.
    let Some(top) = LogicSolver.solve_with_difficulty(puzzle, 1).difficulty else {
        return BottleneckProfile::empty();
    };

    // 2. Fresh state.
    let Some(mut state) = SolveState::new(puzzle) else {
        return BottleneckProfile::empty();
    };

    // 3. Techniques strictly below the top tier drive the cheap closure.
    let below: Vec<_> = TECHNIQUES
        .iter()
        .copied()
        .filter(|t| t.difficulty() < top)
        .collect();

    // 4. Close under `< D` to reach the first stall.
    if !propagate_with(&mut state, &below) {
        return BottleneckProfile::empty();
    }

    // 5. Round loop.
    let mut rounds = Vec::new();
    while !state.is_complete() {
        let keys = find_all_for_tier(&state, top);
        if keys.is_empty() {
            // Shouldn't happen when `top` is the real max tier; guard so a
            // definitional surprise can't spin forever.
            break;
        }
        let width = keys.len();
        for step in &keys {
            if !apply_step(&mut state, step) {
                return BottleneckProfile {
                    top_tier: Some(top),
                    rounds,
                    solved: false,
                };
            }
        }
        let before = placed(&state);
        if !propagate_with(&mut state, &below) {
            return BottleneckProfile {
                top_tier: Some(top),
                rounds,
                solved: false,
            };
        }
        let cascade = placed(&state) - before;
        rounds.push(BottleneckRound { width, cascade });
    }

    let solved = state.is_complete();
    BottleneckProfile {
        top_tier: Some(top),
        rounds,
        solved,
    }
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle};

    use super::*;
    use crate::logic::techniques::{TechniqueResult, apply_technique};

    /// Build a puzzle from a clue specification (mirrors the helper in
    /// `logic.rs` tests).
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

    /// The n=4 seed=13 puzzle from the solver tests: hardest technique is
    /// SimplePermutation (Hard), so it exercises the tier-D round loop.
    fn hard_puzzle_n4() -> Puzzle {
        build_puzzle_with_clues(
            4,
            &[(2, 1, 2), (3, 0, 1)],
            &[(1, 2)],
            &[(3, 2)],
            &[(2, 1)],
            &[],
        )
    }

    #[test]
    fn compute_bottlenecks_on_hard_puzzle() {
        let puzzle = hard_puzzle_n4();
        let profile = compute_bottlenecks(&puzzle);
        assert_eq!(profile.top_tier, Some(Difficulty::Hard));
        assert!(profile.solved, "bottleneck loop should reach completion");
        assert!(
            profile.bottlenecks() >= 1,
            "a Hard puzzle needs at least one top-tier round"
        );
        // Completion requires the final round's cascade to place cells, so at
        // least one round is a release.
        assert!(
            profile.releases() >= 1,
            "a solved Hard puzzle has at least one release round (cascade > 0)"
        );
        // Release + buildup rounds partition the total bottleneck count.
        assert_eq!(
            profile.releases() + profile.buildup_rounds(),
            profile.bottlenecks()
        );
        // Widths are populated and the min-width helper agrees with the rounds.
        assert_eq!(
            profile.min_width(),
            profile.rounds.iter().map(|r| r.width).min()
        );
    }

    #[test]
    fn find_all_matches_apply_availability() {
        // At the first Hard stall, `find_all_for_tier` must surface at least one
        // step whenever the mutating `apply` for that tier would progress.
        let puzzle = hard_puzzle_n4();
        let mut state = SolveState::new(&puzzle).unwrap();
        let below: Vec<_> = TECHNIQUES
            .iter()
            .copied()
            .filter(|t| t.difficulty() < Difficulty::Hard)
            .collect();
        assert!(propagate_with(&mut state, &below));

        let all = find_all_for_tier(&state, Difficulty::Hard);
        assert!(
            !all.is_empty(),
            "expected at least one Hard step available at the stall"
        );
        assert!(
            all.iter()
                .all(|s| s.technique.difficulty() == Difficulty::Hard),
            "find_all_for_tier must only return steps of the requested tier"
        );

        // Cross-check: the greedy dispatch also makes progress here.
        let mut probe = state.clone();
        let progressed = TECHNIQUES
            .iter()
            .filter(|t| t.difficulty() == Difficulty::Hard)
            .any(|&t| matches!(apply_technique(t, &mut probe), TechniqueResult::Progress(_)));
        assert!(progressed);
    }

    #[test]
    fn apply_step_replays_eliminations() {
        // Grab any Hard step and confirm replaying it removes the eliminated
        // candidate from the target cell.
        let puzzle = hard_puzzle_n4();
        let mut state = SolveState::new(&puzzle).unwrap();
        let below: Vec<_> = TECHNIQUES
            .iter()
            .copied()
            .filter(|t| t.difficulty() < Difficulty::Hard)
            .collect();
        assert!(propagate_with(&mut state, &below));

        let steps = find_all_for_tier(&state, Difficulty::Hard);
        let step = steps
            .iter()
            .find(|s| {
                s.actions.iter().any(|a| {
                    matches!(a, crate::logic::difficulty::Action::Eliminate { row, col, value }
                        if state.candidates[state.idx(*row, *col)].contains(*value))
                })
            })
            .expect("expected a Hard step whose elimination is still live");

        assert!(apply_step(&mut state, step));
        for action in &step.actions {
            if let crate::logic::difficulty::Action::Eliminate { row, col, value } = *action {
                assert!(
                    state.grid[state.idx(row, col)].is_some()
                        || !state.candidates[state.idx(row, col)].contains(value),
                    "replayed elimination should be reflected in state"
                );
            }
        }
    }

    #[test]
    fn unsolvable_puzzle_has_empty_profile() {
        // Empty board, no clues — the logic solver cannot solve it.
        let board = Board::new_empty(4);
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let profile = compute_bottlenecks(&puzzle);
        assert_eq!(profile.top_tier, None);
        assert!(!profile.solved);
        assert_eq!(profile.bottlenecks(), 0);
    }
}
