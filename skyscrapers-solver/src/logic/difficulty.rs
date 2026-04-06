/// The difficulty level of a puzzle, determined by the hardest technique required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    /// Solvable with naked singles and hidden singles only.
    Easy,
    /// Requires clue-based elimination.
    Medium,
    /// Requires set-based techniques or X-Wing.
    Hard,
    /// Requires visibility chain reasoning or permutation enumeration.
    Expert,
}

/// Identifies a specific solving technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Technique {
    NakedSingles,
    HiddenSingles,
    CluePruning,
    VisibilityPropagation,
    NakedSets,
    HiddenSets,
    XWing,
    VisibilityChain,
    PermutationEnumeration,
}

impl Technique {
    pub fn difficulty(self) -> Difficulty {
        match self {
            Self::NakedSingles | Self::HiddenSingles => Difficulty::Easy,
            Self::CluePruning | Self::VisibilityPropagation => Difficulty::Medium,
            Self::NakedSets | Self::HiddenSets | Self::XWing => Difficulty::Hard,
            Self::VisibilityChain | Self::PermutationEnumeration => Difficulty::Expert,
        }
    }
}

/// A single action performed by a technique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// A value was placed in a cell.
    Place { row: usize, col: usize, value: u8 },
    /// A candidate was eliminated from a cell.
    Eliminate { row: usize, col: usize, value: u8 },
}

/// One step of logical reasoning (the unit for hints).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub technique: Technique,
    pub actions: Vec<Action>,
    pub reason: Reason,
}

/// The reasoning behind a step (for UI highlighting).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// Cell has only one candidate (Naked Single).
    SingleCandidate { row: usize, col: usize },
    /// Value can only go in one cell within a line (Hidden Single).
    UniqueInLine { line: Line, value: u8 },
    /// Clue constrains candidates.
    ClueConstraint { clue: CluePosition, expected: u8 },
    /// Set-based reasoning within a line (Naked/Hidden Sets).
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
    /// Visibility chain reasoning.
    VisibilityChain {
        clue: CluePosition,
        known_cells: Vec<(usize, usize)>,
    },
    /// Permutation enumeration elimination.
    PermutationElimination { line: Line, clue: CluePosition },
}

/// Identifies a row or column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Line {
    Row(usize),
    Col(usize),
}

/// Identifies a clue position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CluePosition {
    Top(usize),
    Bottom(usize),
    Left(usize),
    Right(usize),
}
