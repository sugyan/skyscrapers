/// The difficulty level of a puzzle, determined by the hardest technique required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Difficulty {
    /// Solvable with naked singles and hidden singles only (init-time
    /// CluePruning may also fire, but does not promote difficulty since
    /// it runs unconditionally from the puzzle's starting clues).
    Easy,
    /// Requires visibility analysis to break out of the singles-only loop.
    Medium,
    /// Requires set-based techniques (NakedSets, X-Wing, ALS-XZ).
    Hard,
    /// Requires permutation enumeration (single- or dual-clue).
    Expert,
    /// Requires forcing-chain reasoning (assumption-based).
    Master,
}

/// Identifies a specific solving technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Technique {
    NakedSingles,
    HiddenSingles,
    CluePruning,
    VisibilityAnalysis,
    NakedSets,
    XWing,
    AlsXz,
    PermutationEnumeration,
    DualCluePermutation,

    SimpleForcingChain,
    FullForcingChain,
}

impl Technique {
    pub fn difficulty(self) -> Difficulty {
        match self {
            Self::NakedSingles | Self::HiddenSingles => Difficulty::Easy,
            Self::CluePruning | Self::VisibilityAnalysis => Difficulty::Medium,
            Self::NakedSets | Self::XWing | Self::AlsXz => Difficulty::Hard,
            Self::PermutationEnumeration | Self::DualCluePermutation => Difficulty::Expert,
            Self::SimpleForcingChain | Self::FullForcingChain => Difficulty::Master,
        }
    }
}

/// A single action performed by a technique.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
pub enum Action {
    /// A value was placed in a cell.
    Place { row: usize, col: usize, value: u8 },
    /// A candidate was eliminated from a cell.
    Eliminate { row: usize, col: usize, value: u8 },
}

/// One step of logical reasoning (the unit for hints).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Step {
    pub technique: Technique,
    pub actions: Vec<Action>,
    pub reason: Reason,
}

/// The reasoning behind a step (for UI highlighting).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
pub enum Reason {
    /// Cell has only one candidate (Naked Single).
    SingleCandidate { row: usize, col: usize },
    /// Value can only go in one cell within a line (Hidden Single).
    UniqueInLine { line: Line, value: u8 },
    /// Set-based reasoning within a line (Naked Sets).
    SetInLine {
        line: Line,
        cells: Vec<(usize, usize)>,
        values: Vec<u8>,
    },
    /// X-Wing / Swordfish pattern.
    FishPattern {
        value: u8,
        base_lines: Vec<Line>,
        cover_lines: Vec<Line>,
    },
    /// Permutation enumeration elimination.
    PermutationElimination { line: Line, clue: CluePosition },
    /// Dual-clue permutation enumeration elimination.
    DualCluePermutationElimination {
        line: Line,
        clue_a: CluePosition,
        clue_b: CluePosition,
    },
    /// ALS-XZ: two almost locked sets connected by a restricted common candidate.
    AlsXzElimination {
        als_a: Vec<(usize, usize)>,
        als_b: Vec<(usize, usize)>,
        rcc_value: u8,
        eliminated_value: u8,
    },
    /// Forcing chain: assuming a value led to a contradiction.
    ForcingChainElimination {
        assumed_cell: (usize, usize),
        assumed_value: u8,
    },
    /// Init-time clue pruning: the clue directly eliminated/placed values
    /// along its line before any solve iteration ran.
    InitialClueConstraint { clue: CluePosition },
    /// Visibility analysis: `n` is placed in the line at position `k` (in
    /// viewing order) and the clue requires `k + 1` visible buildings, so
    /// the `k` cells before `n` are forced to be strictly ascending.
    VisibilityForcing { line: Line, clue: CluePosition },
}

/// Identifies a row or column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Line {
    Row(usize),
    Col(usize),
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
            Self::Expert => "expert",
            Self::Master => "master",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for Difficulty {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            // `grandmaster` accepted as an alias so existing URLs / saved
            // params from the old 6-level scheme still resolve.
            "master" | "grandmaster" => Ok(Self::Master),
            _ => Err(format!("unknown difficulty: {s}")),
        }
    }
}

/// Identifies a clue position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum CluePosition {
    Top(usize),
    Bottom(usize),
    Left(usize),
    Right(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn from_str_accepts_current_labels() {
        assert_eq!(Difficulty::from_str("easy"), Ok(Difficulty::Easy));
        assert_eq!(Difficulty::from_str("medium"), Ok(Difficulty::Medium));
        assert_eq!(Difficulty::from_str("hard"), Ok(Difficulty::Hard));
        assert_eq!(Difficulty::from_str("expert"), Ok(Difficulty::Expert));
        assert_eq!(Difficulty::from_str("master"), Ok(Difficulty::Master));
    }

    #[test]
    fn from_str_legacy_grandmaster_resolves_to_master() {
        // The 6-level scheme had a separate Grandmaster tier; consolidating
        // to 5 levels merged it into Master. Existing URLs / saved params
        // must keep loading the intended bucket.
        assert_eq!(Difficulty::from_str("grandmaster"), Ok(Difficulty::Master));
        assert_eq!(Difficulty::from_str("GRANDMASTER"), Ok(Difficulty::Master));
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!(Difficulty::from_str("trivial").is_err());
        assert!(Difficulty::from_str("").is_err());
    }
}
