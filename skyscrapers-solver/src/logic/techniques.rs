pub(crate) mod als_xz;
pub(crate) mod clue_pruning;

pub(crate) mod dual_clue_permutation;
pub(crate) mod forcing_chain;
pub(crate) mod hidden_singles;
pub(crate) mod naked_sets;
pub(crate) mod naked_singles;
pub(crate) mod permutation;
pub(crate) mod visibility_analysis;
pub(crate) mod x_wing;
pub(crate) mod xy_chain;

#[cfg(feature = "analysis-hooks")]
use super::analysis_hooks;
#[cfg(feature = "analysis-hooks")]
use super::difficulty::{Action, Difficulty};
use super::difficulty::{Step, Technique};
use super::state::SolveState;

/// Result of attempting a technique.
pub(crate) enum TechniqueResult {
    /// Made progress — returns the step describing what was done.
    Progress(Step),
    /// No progress possible with this technique.
    NoProgress,
    /// Detected a contradiction.
    Contradiction,
}

/// The ordered list of techniques to try.
pub(crate) const TECHNIQUES: &[Technique] = &[
    Technique::NakedSingles,
    Technique::HiddenSingles,
    Technique::VisibilityAnalysis,
    Technique::NakedSets,
    Technique::XWing,
    Technique::XyChain,
    Technique::SimplePermutation,
    Technique::AlsXz,
    Technique::PermutationEnumeration,
    Technique::DualCluePermutation,
    Technique::SimpleForcingChain,
    Technique::FullForcingChain,
];

/// Try all techniques in order. Returns the first one that makes progress.
pub(crate) fn apply_next_technique(state: &mut SolveState) -> TechniqueResult {
    for &technique in TECHNIQUES {
        #[cfg(feature = "analysis-hooks")]
        if analysis_hooks::is_disabled(technique) {
            continue;
        }
        let result = apply_technique(technique, state);
        match result {
            TechniqueResult::NoProgress => continue,
            _ => return result,
        }
    }
    TechniqueResult::NoProgress
}

pub(crate) fn apply_technique(technique: Technique, state: &mut SolveState) -> TechniqueResult {
    match technique {
        Technique::NakedSingles => naked_singles::apply(state),
        Technique::HiddenSingles => hidden_singles::apply(state),
        Technique::VisibilityAnalysis => visibility_analysis::apply(state),
        Technique::NakedSets => naked_sets::apply(state),
        Technique::XWing => x_wing::apply(state),
        Technique::XyChain => xy_chain::apply(state),
        Technique::SimplePermutation => permutation::apply_simple(state),
        Technique::AlsXz => als_xz::apply(state),
        Technique::PermutationEnumeration => permutation::apply_complex(state),
        Technique::DualCluePermutation => dual_clue_permutation::apply(state),

        Technique::SimpleForcingChain => forcing_chain::apply_simple(state),
        Technique::FullForcingChain => forcing_chain::apply_full(state),
        // CluePruning runs only once during `SolveState::new` and is never dispatched here.
        Technique::CluePruning => unreachable!("CluePruning is applied during SolveState::new"),
    }
}

/// Run only NakedSingles and HiddenSingles in a loop until no more progress.
/// Used by SimpleForcingChain for basic propagation.
/// Returns false if a contradiction is detected.
pub(crate) fn propagate_simple(state: &mut SolveState) -> bool {
    const SIMPLE_TECHNIQUES: &[Technique] = &[Technique::NakedSingles, Technique::HiddenSingles];
    propagate_with(state, SIMPLE_TECHNIQUES)
}

/// Run all techniques except ForcingChain variants in a loop until no more progress.
/// Used by FullForcingChain for full propagation (no ForcingChain recursion).
/// Returns false if a contradiction is detected.
pub(crate) fn propagate(state: &mut SolveState) -> bool {
    const FULL_TECHNIQUES: &[Technique] = &[
        Technique::NakedSingles,
        Technique::HiddenSingles,
        Technique::VisibilityAnalysis,
        Technique::NakedSets,
        Technique::XWing,
        Technique::XyChain,
        Technique::SimplePermutation,
        Technique::AlsXz,
        Technique::PermutationEnumeration,
        Technique::DualCluePermutation,
    ];
    propagate_with(state, FULL_TECHNIQUES)
}

/// Repeatedly apply the given techniques in order until no technique makes progress.
/// Returns false if a contradiction is detected during propagation, if any unassigned
/// cell has no remaining candidates, or if the grid is complete but violates a clue.
pub(crate) fn propagate_with(state: &mut SolveState, techniques: &[Technique]) -> bool {
    loop {
        let mut progress = false;
        for &technique in techniques {
            #[cfg(feature = "analysis-hooks")]
            if analysis_hooks::is_disabled(technique) {
                continue;
            }
            match apply_technique(technique, state) {
                TechniqueResult::Contradiction => return false,
                TechniqueResult::Progress(_) => {
                    progress = true;
                    break;
                }
                TechniqueResult::NoProgress => continue,
            }
        }
        if !progress {
            break;
        }
    }
    for idx in 0..state.n * state.n {
        if state.grid[idx].is_none() && state.candidates[idx].is_empty() {
            return false;
        }
    }
    if state.is_complete() && !state.verify_clues() {
        return false;
    }
    true
}

/// Enumerate every step available at `state` for techniques of exactly `tier`,
/// without mutating `state`.
///
/// Implemented by *clone-saturation*: for each technique in that tier, clone the
/// state and drive its (mutating) `apply` in a loop until it stops making
/// progress, collecting each produced `Step`. Because every technique is a
/// monotone, sound elimination, the collected steps are all valid at `state` and
/// can be replayed onto it (see [`apply_step`]). Used only by the dev-only
/// bottleneck metric; keep it off the early-terminating solve/hint hot paths.
#[cfg(feature = "analysis-hooks")]
pub(crate) fn find_all_for_tier(state: &SolveState, tier: Difficulty) -> Vec<Step> {
    let mut steps = Vec::new();
    for &technique in TECHNIQUES {
        if technique.difficulty() != tier {
            continue;
        }
        let mut clone = state.clone();
        while let TechniqueResult::Progress(step) = apply_technique(technique, &mut clone) {
            steps.push(step);
        }
    }
    steps
}

/// Replay a `Step`'s actions onto `state`. Returns false on contradiction.
///
/// Sound steps produced by [`find_all_for_tier`] never empty a cell's candidate
/// set, so this succeeds in practice; the bool is a defensive guard.
#[cfg(feature = "analysis-hooks")]
pub(crate) fn apply_step(state: &mut SolveState, step: &Step) -> bool {
    for action in &step.actions {
        let ok = match *action {
            Action::Place { row, col, value } => state.assign(row, col, value),
            Action::Eliminate { row, col, value } => state.eliminate(row, col, value),
        };
        if !ok {
            return false;
        }
    }
    true
}
