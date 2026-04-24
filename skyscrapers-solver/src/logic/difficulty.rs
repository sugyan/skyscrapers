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
    /// Requires simple forcing chain (basic propagation only).
    Master,
    /// Requires full forcing chain (all techniques in propagation).
    Grandmaster,
}

/// Identifies a specific solving technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Technique {
    NakedSingles,
    HiddenSingles,
    CluePruning,
    VisibilityAnalysis,
    NakedSets,
    HiddenSets,
    XWing,
    XYWing,
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
            Self::NakedSets | Self::HiddenSets | Self::XWing | Self::XYWing | Self::AlsXz => {
                Difficulty::Hard
            }
            Self::PermutationEnumeration | Self::DualCluePermutation => Difficulty::Expert,
            Self::SimpleForcingChain => Difficulty::Master,
            Self::FullForcingChain => Difficulty::Grandmaster,
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
    /// Permutation enumeration elimination.
    PermutationElimination { line: Line, clue: CluePosition },
    /// Dual-clue permutation enumeration elimination.
    DualCluePermutationElimination {
        line: Line,
        clue_a: CluePosition,
        clue_b: CluePosition,
    },
    /// XY-Wing: three bivalue cells eliminate a candidate.
    XYWingElimination {
        pivot: (usize, usize),
        wing_a: (usize, usize),
        wing_b: (usize, usize),
        eliminated_value: u8,
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
            Self::Grandmaster => "grandmaster",
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
            "master" => Ok(Self::Master),
            "grandmaster" => Ok(Self::Grandmaster),
            _ => Err(format!("unknown difficulty: {s}")),
        }
    }
}

/// Identifies a clue position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CluePosition {
    Top(usize),
    Bottom(usize),
    Left(usize),
    Right(usize),
}
