#[cfg(target_arch = "wasm32")]
mod wasm;

use latin_sampler::LatinSquare;
use rand::seq::SliceRandom;
use skyscrapers_core::{Board, Clues, Puzzle, Solution};
use skyscrapers_solver::{BacktrackingSolver, Difficulty, LogicSolver, Solver};

/// Converts a `LatinSquare` (0-based symbols) into a `Solution` (1-based heights).
pub fn solution_from_latin_square(ls: &LatinSquare) -> Solution {
    let n = ls.n();
    let cells: Vec<Vec<u8>> = (0..n)
        .map(|r| (0..n).map(|c| ls.get(r, c) + 1).collect())
        .collect();
    Solution::new(n, cells)
}

/// Derives all clues from a solution.
pub fn derive_clues(solution: &Solution) -> Clues {
    Clues::from_solution(solution)
}

/// Creates a fully filled `Board` from a `Solution`.
fn board_from_solution(sol: &Solution) -> Board {
    let n = sol.n();
    let mut board = Board::new_empty(n);
    for r in 0..n {
        for c in 0..n {
            board.set(r, c, Some(sol.get(r, c)));
        }
    }
    board
}

/// A removal target: either a board cell or a clue.
#[derive(Debug, Clone, Copy)]
enum RemovalTarget {
    Cell(usize, usize),
    ClueTop(usize),
    ClueBottom(usize),
    ClueLeft(usize),
    ClueRight(usize),
}

/// Gets the clue value for a clue removal target.
fn get_clue(clues: &Clues, target: RemovalTarget) -> Option<u8> {
    match target {
        RemovalTarget::ClueTop(i) => clues.top(i),
        RemovalTarget::ClueBottom(i) => clues.bottom(i),
        RemovalTarget::ClueLeft(i) => clues.left(i),
        RemovalTarget::ClueRight(i) => clues.right(i),
        RemovalTarget::Cell(..) => unreachable!(),
    }
}

/// Sets a clue value for a clue removal target.
fn set_clue(clues: &mut Clues, target: RemovalTarget, value: Option<u8>) {
    match target {
        RemovalTarget::ClueTop(i) => clues.set_top(i, value),
        RemovalTarget::ClueBottom(i) => clues.set_bottom(i, value),
        RemovalTarget::ClueLeft(i) => clues.set_left(i, value),
        RemovalTarget::ClueRight(i) => clues.set_right(i, value),
        RemovalTarget::Cell(..) => unreachable!(),
    }
}

/// Checks whether a removal is acceptable.
///
/// When `target_difficulty` is `None`, only uniqueness is checked via `solver`.
/// When `target_difficulty` is `Some(target)`, uniqueness is checked first,
/// then the logic solver verifies the puzzle is solvable within the target difficulty.
fn is_removal_ok(
    puzzle: &Puzzle,
    solver: &dyn Solver,
    target_difficulty: Option<Difficulty>,
) -> bool {
    if solver.solve(puzzle, 2).len() != 1 {
        return false;
    }
    if let Some(target) = target_difficulty {
        let result = LogicSolver.solve_with_difficulty(puzzle, 1);
        matches!(result.difficulty, Some(d) if d <= target)
    } else {
        true
    }
}

/// Greedy removal of board cells and clues while preserving uniqueness.
///
/// Strategy: remove board cells first, then clues.
/// Each removal is tested by temporarily clearing the value and checking
/// acceptability. If the removal breaks uniqueness or exceeds the target
/// difficulty, the original value is restored.
fn greedy_remove<R: rand::Rng>(
    rng: &mut R,
    board: Board,
    clues: Clues,
    solver: &dyn Solver,
    target_difficulty: Option<Difficulty>,
) -> (Puzzle, Option<Difficulty>) {
    let n = board.n();
    let mut puzzle = Puzzle { board, clues };

    // Phase 1: Remove board cells
    let mut cell_targets: Vec<RemovalTarget> = (0..n)
        .flat_map(|r| (0..n).map(move |c| RemovalTarget::Cell(r, c)))
        .collect();
    cell_targets.shuffle(rng);

    for target in &cell_targets {
        let RemovalTarget::Cell(r, c) = *target else {
            unreachable!()
        };
        let saved = puzzle.board.get(r, c);
        if saved.is_none() {
            continue;
        }
        puzzle.board.set(r, c, None);
        if !is_removal_ok(&puzzle, solver, target_difficulty) {
            puzzle.board.set(r, c, saved);
        }
    }

    // Phase 2: Remove clues
    let mut clue_targets: Vec<RemovalTarget> = (0..n)
        .flat_map(|i| {
            [
                RemovalTarget::ClueTop(i),
                RemovalTarget::ClueBottom(i),
                RemovalTarget::ClueLeft(i),
                RemovalTarget::ClueRight(i),
            ]
        })
        .collect();
    clue_targets.shuffle(rng);

    for target in &clue_targets {
        let saved = get_clue(&puzzle.clues, *target);
        if saved.is_none() {
            continue;
        }
        set_clue(&mut puzzle.clues, *target, None);
        if !is_removal_ok(&puzzle, solver, target_difficulty) {
            set_clue(&mut puzzle.clues, *target, saved);
        }
    }

    let difficulty = if target_difficulty.is_some() {
        LogicSolver.solve_with_difficulty(&puzzle, 1).difficulty
    } else {
        None
    };

    (puzzle, difficulty)
}

/// Error returned when puzzle generation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateError {
    /// Could not generate a puzzle at the target difficulty within the attempt limit.
    MaxAttemptsExceeded { attempts: usize },
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxAttemptsExceeded { attempts } => {
                write!(
                    f,
                    "failed to generate puzzle at target difficulty after {attempts} attempts"
                )
            }
        }
    }
}

impl std::error::Error for GenerateError {}

/// Parameters for puzzle generation.
pub struct GeneratorParams {
    pub n: usize,
    pub solver: Box<dyn Solver>,
    pub sampler_params: latin_sampler::SamplerParams,
    pub target_difficulty: Option<Difficulty>,
    pub max_attempts: usize,
}

impl GeneratorParams {
    /// Creates new generator parameters with default settings.
    ///
    /// Uses `BacktrackingSolver` for uniqueness checking, default sampler params,
    /// no target difficulty, and 100 max attempts.
    ///
    /// # Panics
    /// Panics if `n` is not in `1..=9`.
    pub fn new(n: usize) -> Self {
        assert!((1..=9).contains(&n), "n must be in range 1..=9");
        Self {
            n,
            solver: Box::new(BacktrackingSolver),
            sampler_params: latin_sampler::SamplerParams::default(),
            target_difficulty: None,
            max_attempts: 100,
        }
    }

    pub fn with_solver(mut self, solver: impl Solver + 'static) -> Self {
        self.solver = Box::new(solver);
        self
    }

    pub fn with_target_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.target_difficulty = Some(difficulty);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }
}

/// Generates a Skyscrapers puzzle with a guaranteed unique solution.
///
/// Returns the puzzle and its solution.
///
/// When `target_difficulty` is set, retries with different latin squares
/// until a puzzle matching the target difficulty is produced, or returns
/// an error after `max_attempts`.
///
/// Pipeline: sample latin square → convert to solution → derive board + clues
/// → greedy removal → (puzzle, solution)
pub fn generate<R: rand::Rng>(
    rng: &mut R,
    params: &GeneratorParams,
) -> Result<(Puzzle, Solution), GenerateError> {
    let attempts = if params.target_difficulty.is_some() {
        params.max_attempts
    } else {
        1
    };

    for _ in 0..attempts {
        let ls = latin_sampler::sample(params.n, rng, &params.sampler_params);
        let solution = solution_from_latin_square(&ls);
        let board = board_from_solution(&solution);
        let clues = derive_clues(&solution);
        let (puzzle, difficulty) = greedy_remove(
            rng,
            board,
            clues,
            params.solver.as_ref(),
            params.target_difficulty,
        );

        match params.target_difficulty {
            None => return Ok((puzzle, solution)),
            Some(target) if difficulty == Some(target) => return Ok((puzzle, solution)),
            _ => continue,
        }
    }

    Err(GenerateError::MaxAttemptsExceeded {
        attempts: params.max_attempts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use latin_sampler::SamplerParams;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn sample_latin_square(n: usize, seed: u64) -> LatinSquare {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        latin_sampler::sample(n, &mut rng, &SamplerParams::default())
    }

    #[test]
    fn solution_from_latin_square_converts_to_1_based() {
        let ls = sample_latin_square(4, 42);
        let sol = solution_from_latin_square(&ls);

        assert_eq!(sol.n(), 4);
        for r in 0..4 {
            for c in 0..4 {
                let v = sol.get(r, c);
                assert!((1..=4).contains(&v), "cell ({r},{c}) = {v}, expected 1..=4");
                assert_eq!(v, ls.get(r, c) + 1);
            }
        }
    }

    #[test]
    fn solution_is_valid_latin_square() {
        let ls = sample_latin_square(5, 123);
        let sol = solution_from_latin_square(&ls);
        let n = sol.n();

        for r in 0..n {
            let mut row: Vec<u8> = (0..n).map(|c| sol.get(r, c)).collect();
            row.sort();
            assert_eq!(row, (1..=n as u8).collect::<Vec<_>>());
        }
        for c in 0..n {
            let mut col: Vec<u8> = (0..n).map(|r| sol.get(r, c)).collect();
            col.sort();
            assert_eq!(col, (1..=n as u8).collect::<Vec<_>>());
        }
    }

    #[test]
    fn derive_clues_all_present() {
        let ls = sample_latin_square(4, 42);
        let sol = solution_from_latin_square(&ls);
        let clues = derive_clues(&sol);

        assert_eq!(clues.n(), 4);
        for i in 0..4 {
            for v in [clues.top(i), clues.bottom(i), clues.left(i), clues.right(i)] {
                assert!(v.is_some());
                let v = v.unwrap();
                assert!((1..=4).contains(&v), "clue value {v} out of range");
            }
        }
    }

    #[test]
    fn derive_clues_known_row() {
        let sol = Solution::new(
            4,
            vec![
                vec![2, 1, 4, 3],
                vec![3, 4, 1, 2],
                vec![4, 3, 2, 1],
                vec![1, 2, 3, 4],
            ],
        );
        let clues = derive_clues(&sol);

        assert_eq!(clues.left(3), Some(4));
        assert_eq!(clues.right(2), Some(4));
        assert_eq!(clues.left(2), Some(1));
        assert_eq!(clues.right(3), Some(1));
    }

    #[test]
    fn board_from_solution_fills_all_cells() {
        let sol = Solution::new(3, vec![vec![1, 2, 3], vec![2, 3, 1], vec![3, 1, 2]]);
        let board = board_from_solution(&sol);
        assert_eq!(board.n(), 3);
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(board.get(r, c), Some(sol.get(r, c)));
            }
        }
    }

    fn make_generator_params(n: usize) -> GeneratorParams {
        GeneratorParams::new(n)
    }

    #[test]
    fn generate_produces_unique_solution() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let params = make_generator_params(4);
        let (puzzle, _solution) = generate(&mut rng, &params).unwrap();

        let solutions = BacktrackingSolver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn generate_removes_some_cells_and_clues() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let params = make_generator_params(4);
        let (puzzle, _solution) = generate(&mut rng, &params).unwrap();
        let n = puzzle.board.n();

        let mut given_count = 0;
        for r in 0..n {
            for c in 0..n {
                if puzzle.board.get(r, c).is_some() {
                    given_count += 1;
                }
            }
        }
        assert!(
            given_count < n * n,
            "expected some cells to be removed, but all {given_count} cells are given"
        );

        let mut clue_count = 0;
        for i in 0..n {
            if puzzle.clues.top(i).is_some() {
                clue_count += 1;
            }
            if puzzle.clues.bottom(i).is_some() {
                clue_count += 1;
            }
            if puzzle.clues.left(i).is_some() {
                clue_count += 1;
            }
            if puzzle.clues.right(i).is_some() {
                clue_count += 1;
            }
        }
        assert!(
            clue_count < 4 * n,
            "expected some clues to be removed, but all {clue_count} clues are present"
        );
    }

    #[test]
    fn generate_deterministic_with_seed() {
        let params = make_generator_params(4);

        let (puzzle1, sol1) = generate(&mut ChaCha20Rng::seed_from_u64(99), &params).unwrap();
        let (puzzle2, sol2) = generate(&mut ChaCha20Rng::seed_from_u64(99), &params).unwrap();

        assert_eq!(puzzle1, puzzle2);
        assert_eq!(sol1, sol2);
    }

    #[test]
    fn logic_solver_solution_matches_backtracking_when_solvable() {
        use skyscrapers_solver::LogicSolver;

        let params = make_generator_params(5);
        // Test multiple seeds to cover different puzzle configurations
        for seed in 0..10 {
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let (puzzle, expected_solution) = generate(&mut rng, &params).unwrap();

            let bt_solutions = BacktrackingSolver.solve(&puzzle, 2);
            assert_eq!(
                bt_solutions.len(),
                1,
                "seed {seed}: BT should find exactly 1 solution"
            );
            assert_eq!(
                bt_solutions[0], expected_solution,
                "seed {seed}: BT solution mismatch"
            );

            let logic_result = LogicSolver.solve_with_difficulty(&puzzle, 1);
            if !logic_result.solutions.is_empty() {
                let logic_sol = &logic_result.solutions[0];
                let n = puzzle.board.n();
                // Verify the logic solver's solution is actually valid:
                // it must be a valid Latin square that satisfies all clues
                let derived = Clues::from_solution(logic_sol);
                for idx in 0..n {
                    if let Some(c) = puzzle.clues.top(idx) {
                        assert_eq!(
                            derived.top(idx),
                            Some(c),
                            "seed {seed}: Logic solver solution violates top clue at col {idx}"
                        );
                    }
                    if let Some(c) = puzzle.clues.bottom(idx) {
                        assert_eq!(
                            derived.bottom(idx),
                            Some(c),
                            "seed {seed}: Logic solver solution violates bottom clue at col {idx}"
                        );
                    }
                    if let Some(c) = puzzle.clues.left(idx) {
                        assert_eq!(
                            derived.left(idx),
                            Some(c),
                            "seed {seed}: Logic solver solution violates left clue at row {idx}"
                        );
                    }
                    if let Some(c) = puzzle.clues.right(idx) {
                        assert_eq!(
                            derived.right(idx),
                            Some(c),
                            "seed {seed}: Logic solver solution violates right clue at row {idx}"
                        );
                    }
                }
                // Also verify board cells are respected
                for r in 0..n {
                    for c in 0..n {
                        if let Some(v) = puzzle.board.get(r, c) {
                            assert_eq!(
                                logic_sol.get(r, c),
                                v,
                                "seed {seed}: Logic solver doesn't match board at ({r},{c})"
                            );
                        }
                    }
                }
                // Double-check uniqueness with higher limit
                let all_solutions = BacktrackingSolver.solve(&puzzle, 10);
                assert_eq!(
                    all_solutions.len(),
                    1,
                    "seed {seed}: BT finds {} solutions, puzzle not unique!\nPuzzle:\n{puzzle}\nLogic sol: {:?}\nBT sol: {:?}",
                    all_solutions.len(),
                    logic_sol,
                    expected_solution
                );
                // Must match the expected unique solution
                assert_eq!(
                    *logic_sol, expected_solution,
                    "seed {seed}: Logic solver found different solution"
                );
                assert!(
                    logic_result.difficulty.is_some(),
                    "seed {seed}: Difficulty should be reported"
                );
            }
            // If logic solver can't solve it, that's OK — it just means the puzzle
            // requires techniques not yet implemented
        }
    }

    #[test]
    fn generate_with_target_difficulty_easy() {
        let params = GeneratorParams::new(4)
            .with_target_difficulty(Difficulty::Easy)
            .with_max_attempts(50);
        let mut rng = ChaCha20Rng::seed_from_u64(0);
        let (puzzle, _solution) = generate(&mut rng, &params).expect("should find an Easy puzzle");
        let result = LogicSolver.solve_with_difficulty(&puzzle, 1);
        assert_eq!(result.difficulty, Some(Difficulty::Easy));
        assert_eq!(result.solutions.len(), 1);
    }

    #[test]
    fn generate_with_target_difficulty_unique_solution() {
        let params = GeneratorParams::new(5)
            .with_target_difficulty(Difficulty::Hard)
            .with_max_attempts(50);
        let mut rng = ChaCha20Rng::seed_from_u64(0);
        if let Ok((puzzle, solution)) = generate(&mut rng, &params) {
            let bt_solutions = BacktrackingSolver.solve(&puzzle, 2);
            assert_eq!(bt_solutions.len(), 1);
            assert_eq!(bt_solutions[0], solution);
        }
    }

    #[test]
    fn generate_with_target_difficulty_deterministic() {
        let params = GeneratorParams::new(4).with_target_difficulty(Difficulty::Easy);
        let result1 = generate(&mut ChaCha20Rng::seed_from_u64(0), &params);
        let result2 = generate(&mut ChaCha20Rng::seed_from_u64(0), &params);
        assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok((p1, s1)), Ok((p2, s2))) = (result1, result2) {
            assert_eq!(p1, p2);
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn generate_without_target_difficulty_unchanged() {
        let params = make_generator_params(4);
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let result = generate(&mut rng, &params);
        assert!(result.is_ok());
    }
}
