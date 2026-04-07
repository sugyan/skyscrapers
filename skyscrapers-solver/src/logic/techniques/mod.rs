pub(crate) mod clue_pruning;
pub(crate) mod hidden_sets;
pub(crate) mod hidden_singles;
pub(crate) mod naked_sets;
pub(crate) mod naked_singles;
pub(crate) mod permutation;
pub(crate) mod x_wing;

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
const TECHNIQUES: &[Technique] = &[
    Technique::NakedSingles,
    Technique::HiddenSingles,
    Technique::NakedSets,
    Technique::HiddenSets,
    Technique::XWing,
    Technique::PermutationEnumeration,
];

/// Try all techniques in order. Returns the first one that makes progress.
pub(crate) fn apply_next_technique(state: &mut SolveState) -> TechniqueResult {
    for &technique in TECHNIQUES {
        let result = apply_technique(technique, state);
        match result {
            TechniqueResult::NoProgress => continue,
            _ => return result,
        }
    }
    TechniqueResult::NoProgress
}

fn apply_technique(technique: Technique, state: &mut SolveState) -> TechniqueResult {
    match technique {
        Technique::NakedSingles => naked_singles::apply(state),
        Technique::HiddenSingles => hidden_singles::apply(state),
        Technique::NakedSets => naked_sets::apply(state),
        Technique::HiddenSets => hidden_sets::apply(state),
        Technique::XWing => x_wing::apply(state),
        Technique::PermutationEnumeration => permutation::apply(state),
        Technique::CluePruning => TechniqueResult::NoProgress, // applied during initialization
    }
}
