#[cfg(target_arch = "wasm32")]
mod wasm;

use latin_sampler::LatinSquare;
use rand::seq::SliceRandom;
use skyscrapers_core::{Board, Clues, Puzzle, Solution};
use skyscrapers_solver::Solver;

/// Converts a `LatinSquare` (0-based symbols) into a `Solution` (1-based heights).
pub fn solution_from_latin_square(ls: &LatinSquare) -> Solution {
    let n = ls.n();
    let cells: Vec<u8> = ls.cells().iter().map(|&v| v + 1).collect();
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

/// Greedy removal of board cells and clues while preserving uniqueness.
///
/// Strategy: remove board cells first, then clues.
/// Each removal is tested by temporarily clearing the value and checking
/// uniqueness. If uniqueness is lost, the original value is restored.
///
/// NOTE: This two-phase strategy may be changed in the future to a mixed
/// strategy where board cells and clues are interleaved randomly.
fn greedy_remove<R: rand::Rng>(
    rng: &mut R,
    board: Board,
    clues: Clues,
    solver: &dyn Solver,
) -> Puzzle {
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
        if solver.solve(&puzzle, 2).len() != 1 {
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
        if solver.solve(&puzzle, 2).len() != 1 {
            set_clue(&mut puzzle.clues, *target, saved);
        }
    }

    puzzle
}

/// Parameters for puzzle generation.
pub struct GeneratorParams {
    pub n: usize,
    pub solver: Box<dyn Solver>,
    pub sampler_params: latin_sampler::SamplerParams,
}

impl GeneratorParams {
    /// Creates new generator parameters with default sampler settings.
    ///
    /// # Panics
    /// Panics if `n` is not in `1..=9`.
    pub fn new(n: usize, solver: impl Solver + 'static) -> Self {
        assert!((1..=9).contains(&n), "n must be in range 1..=9");
        Self {
            n,
            solver: Box::new(solver),
            sampler_params: latin_sampler::SamplerParams::default(),
        }
    }
}

/// Generates a Skyscrapers puzzle with a guaranteed unique solution.
///
/// Pipeline: sample latin square → convert to solution → derive board + clues
/// → greedy removal → puzzle
pub fn generate<R: rand::Rng>(rng: &mut R, params: &GeneratorParams) -> Puzzle {
    let ls = latin_sampler::sample(params.n, rng, &params.sampler_params);
    let solution = solution_from_latin_square(&ls);
    let board = board_from_solution(&solution);
    let clues = derive_clues(&solution);
    greedy_remove(rng, board, clues, params.solver.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use latin_sampler::SamplerParams;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use skyscrapers_solver::BacktrackingSolver;

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
                assert!(v >= 1 && v <= 4, "cell ({r},{c}) = {v}, expected 1..=4");
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
                assert!(v >= 1 && v <= 4, "clue value {v} out of range");
            }
        }
    }

    #[test]
    fn derive_clues_known_row() {
        let sol = Solution::new(
            4,
            vec![
                2, 1, 4, 3, //
                3, 4, 1, 2, //
                4, 3, 2, 1, //
                1, 2, 3, 4, //
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
        let sol = Solution::new(3, vec![1, 2, 3, 2, 3, 1, 3, 1, 2]);
        let board = board_from_solution(&sol);
        assert_eq!(board.n(), 3);
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(board.get(r, c), Some(sol.get(r, c)));
            }
        }
    }

    fn make_generator_params(n: usize) -> GeneratorParams {
        GeneratorParams {
            n,
            solver: Box::new(BacktrackingSolver),
            sampler_params: SamplerParams::default(),
        }
    }

    #[test]
    fn generate_produces_unique_solution() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let params = make_generator_params(4);
        let puzzle = generate(&mut rng, &params);

        let solutions = BacktrackingSolver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn generate_removes_some_cells_and_clues() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let params = make_generator_params(4);
        let puzzle = generate(&mut rng, &params);
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

        let puzzle1 = generate(&mut ChaCha20Rng::seed_from_u64(99), &params);
        let puzzle2 = generate(&mut ChaCha20Rng::seed_from_u64(99), &params);

        assert_eq!(puzzle1, puzzle2);
    }
}
