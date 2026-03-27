use skyscrapers_core::{Puzzle, Solution};
use varisat::ExtendFormula;
use varisat::Lit;
use varisat::Var;

use crate::Solver;

/// A SAT-based solver using the varisat CDCL solver.
///
/// Encodes the Skyscrapers puzzle as a boolean satisfiability problem:
/// - Cell-value variables for each (row, col, value) triple
/// - Latin square constraints (exactly-one per cell, row, column)
/// - Given cell constraints (unit clauses)
/// - Clue constraints via permutation enumeration + Tseitin transformation
pub struct SatSolver;

impl Solver for SatSolver {
    fn solve(&self, puzzle: &Puzzle, limit: usize) -> Vec<Solution> {
        if limit == 0 {
            return Vec::new();
        }
        let n = puzzle.board.n();
        if n == 0 || n > 9 || puzzle.clues.n() != n {
            return Vec::new();
        }

        let mut solver = varisat::Solver::new();
        let mut next_var_index = n * n * n;

        add_latin_square_clauses(&mut solver, n);
        add_given_clauses(&mut solver, n, &puzzle.board);
        add_clue_clauses(&mut solver, n, &puzzle.clues, &mut next_var_index);

        // Find up to `limit` solutions using blocking clauses
        let mut solutions = Vec::new();
        while solutions.len() < limit {
            match solver.solve() {
                Ok(true) => {
                    let model = solver.model().unwrap();
                    let solution = decode_model(&model, n);
                    // Add blocking clause: at least one cell must differ
                    let blocking: Vec<Lit> = (0..n * n)
                        .map(|idx| {
                            let r = idx / n;
                            let c = idx % n;
                            let v = solution.get(r, c) as usize - 1;
                            cell_lit(n, r, c, v, false)
                        })
                        .collect();
                    solver.add_clause(&blocking);
                    solutions.push(solution);
                }
                _ => break,
            }
        }
        solutions
    }
}

/// Map (row, col, value) to a varisat variable.
/// `v` is 0-indexed (represents value v+1).
fn cell_var(n: usize, r: usize, c: usize, v: usize) -> Var {
    Var::from_index(r * n * n + c * n + v)
}

/// Create a literal for cell (r, c) having value v+1.
fn cell_lit(n: usize, r: usize, c: usize, v: usize, positive: bool) -> Lit {
    Lit::from_var(cell_var(n, r, c, v), positive)
}

/// Add Latin square constraints: each cell has exactly one value,
/// each value appears exactly once per row and column.
fn add_latin_square_clauses(solver: &mut varisat::Solver, n: usize) {
    for r in 0..n {
        for c in 0..n {
            // At-least-one: cell (r,c) has some value
            let alo: Vec<Lit> = (0..n).map(|v| cell_lit(n, r, c, v, true)).collect();
            solver.add_clause(&alo);

            // At-most-one: no two values in the same cell
            for v1 in 0..n {
                for v2 in (v1 + 1)..n {
                    solver
                        .add_clause(&[cell_lit(n, r, c, v1, false), cell_lit(n, r, c, v2, false)]);
                }
            }
        }
    }

    // Row uniqueness
    for r in 0..n {
        for v in 0..n {
            // At-least-one in row
            let alo: Vec<Lit> = (0..n).map(|c| cell_lit(n, r, c, v, true)).collect();
            solver.add_clause(&alo);

            // At-most-one in row
            for c1 in 0..n {
                for c2 in (c1 + 1)..n {
                    solver
                        .add_clause(&[cell_lit(n, r, c1, v, false), cell_lit(n, r, c2, v, false)]);
                }
            }
        }
    }

    // Column uniqueness
    for c in 0..n {
        for v in 0..n {
            // At-least-one in column
            let alo: Vec<Lit> = (0..n).map(|r| cell_lit(n, r, c, v, true)).collect();
            solver.add_clause(&alo);

            // At-most-one in column
            for r1 in 0..n {
                for r2 in (r1 + 1)..n {
                    solver
                        .add_clause(&[cell_lit(n, r1, c, v, false), cell_lit(n, r2, c, v, false)]);
                }
            }
        }
    }
}

/// Add unit clauses for pre-filled board cells.
fn add_given_clauses(solver: &mut varisat::Solver, n: usize, board: &skyscrapers_core::Board) {
    for r in 0..n {
        for c in 0..n {
            if let Some(v) = board.get(r, c) {
                solver.add_clause(&[cell_lit(n, r, c, v as usize - 1, true)]);
            }
        }
    }
}

/// Add clue constraints using permutation enumeration and Tseitin transformation.
fn add_clue_clauses(
    solver: &mut varisat::Solver,
    n: usize,
    clues: &skyscrapers_core::Clues,
    next_var_index: &mut usize,
) {
    // Row clues (left/right)
    for r in 0..n {
        let clue_left = clues.left(r);
        let clue_right = clues.right(r);
        if clue_left.is_none() && clue_right.is_none() {
            continue;
        }

        let perms = valid_permutations(n, clue_left, clue_right);
        if perms.is_empty() {
            // Contradiction: no valid permutation — add empty clause
            solver.add_clause(&[]);
            return;
        }

        // Cell indices for this row: (r, 0), (r, 1), ..., (r, n-1)
        add_permutation_clauses(solver, n, &perms, next_var_index, |pos, v| {
            cell_lit(n, r, pos, v, true)
        });
    }

    // Column clues (top/bottom)
    for c in 0..n {
        let clue_top = clues.top(c);
        let clue_bottom = clues.bottom(c);
        if clue_top.is_none() && clue_bottom.is_none() {
            continue;
        }

        let perms = valid_permutations(n, clue_top, clue_bottom);
        if perms.is_empty() {
            solver.add_clause(&[]);
            return;
        }

        // Cell indices for this column: (0, c), (1, c), ..., (n-1, c)
        add_permutation_clauses(solver, n, &perms, next_var_index, |pos, v| {
            cell_lit(n, pos, c, v, true)
        });
    }
}

/// Add Tseitin-encoded permutation constraints.
///
/// For each valid permutation p_j, introduce auxiliary variable a_j:
/// - a_j => x[pos][p_j[pos]] for all positions (implication clauses)
/// - at least one a_j must be true (selection clause)
fn add_permutation_clauses(
    solver: &mut varisat::Solver,
    _n: usize,
    perms: &[Vec<u8>],
    next_var_index: &mut usize,
    cell_lit_fn: impl Fn(usize, usize) -> Lit,
) {
    let base = *next_var_index;
    *next_var_index += perms.len();

    // For each permutation, add implication clauses
    let mut selection_clause = Vec::with_capacity(perms.len());

    for (j, perm) in perms.iter().enumerate() {
        let aux_var = Var::from_index(base + j);
        let aux_pos = Lit::positive(aux_var);
        let aux_neg = Lit::negative(aux_var);

        selection_clause.push(aux_pos);

        // a_j => x[pos][perm[pos]-1] for each position
        for (pos, &val) in perm.iter().enumerate() {
            let x_lit = cell_lit_fn(pos, val as usize - 1);
            solver.add_clause(&[aux_neg, x_lit]);
        }
    }

    // At least one permutation must be selected
    solver.add_clause(&selection_clause);

    // Hint: for single-permutation case, the auxiliary is forced true,
    // so the implication clauses effectively become unit propagations.
    // No additional optimization needed — the SAT solver handles this.
}

/// Enumerate all permutations of 1..=n that satisfy the given clue constraints.
/// `clue_fwd` is the visibility count from the forward direction (index 0 first).
/// `clue_rev` is the visibility count from the reverse direction (index n-1 first).
fn valid_permutations(n: usize, clue_fwd: Option<u8>, clue_rev: Option<u8>) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut perm = vec![0u8; n];
    let mut used = vec![false; n + 1]; // 1-indexed
    generate_permutations(n, 0, &mut perm, &mut used, clue_fwd, clue_rev, &mut result);
    result
}

/// Recursive permutation generator with early pruning.
fn generate_permutations(
    n: usize,
    pos: usize,
    perm: &mut Vec<u8>,
    used: &mut Vec<bool>,
    clue_fwd: Option<u8>,
    clue_rev: Option<u8>,
    result: &mut Vec<Vec<u8>>,
) {
    if pos == n {
        // Check forward clue
        if let Some(expected) = clue_fwd {
            if count_visible(perm) != expected {
                return;
            }
        }
        // Check reverse clue
        if let Some(expected) = clue_rev {
            if count_visible_rev(perm) != expected {
                return;
            }
        }
        result.push(perm.clone());
        return;
    }

    for v in 1..=n as u8 {
        if !used[v as usize] {
            perm[pos] = v;
            used[v as usize] = true;
            generate_permutations(n, pos + 1, perm, used, clue_fwd, clue_rev, result);
            used[v as usize] = false;
        }
    }
}

/// Count visible buildings from the front (index 0).
fn count_visible(perm: &[u8]) -> u8 {
    let mut max = 0u8;
    let mut count = 0u8;
    for &h in perm {
        if h > max {
            count += 1;
            max = h;
        }
    }
    count
}

/// Count visible buildings from the back (index n-1).
fn count_visible_rev(perm: &[u8]) -> u8 {
    let mut max = 0u8;
    let mut count = 0u8;
    for &h in perm.iter().rev() {
        if h > max {
            count += 1;
            max = h;
        }
    }
    count
}

/// Decode a SAT model into a Solution.
fn decode_model(model: &[Lit], n: usize) -> Solution {
    let mut cells = vec![0u8; n * n];
    for &lit in model {
        if lit.is_positive() {
            let idx = lit.index();
            // Only look at cell-value variables (indices 0..n³)
            if idx < n * n * n {
                let v = idx % n;
                let remaining = idx / n;
                let c = remaining % n;
                let r = remaining / n;
                cells[r * n + c] = v as u8 + 1;
            }
        }
    }
    Solution::new(n, cells)
}

#[cfg(test)]
mod tests {
    use skyscrapers_core::{Board, Clues, Puzzle, Solution};

    use super::*;

    fn make_4x4_solution() -> Solution {
        Solution::new(
            4,
            vec![
                2, 1, 4, 3, //
                3, 4, 1, 2, //
                4, 3, 2, 1, //
                1, 2, 3, 4, //
            ],
        )
    }

    fn make_4x4_puzzle_full() -> Puzzle {
        let sol = make_4x4_solution();
        let clues = Clues::from_solution(&sol);
        let mut board = Board::new_empty(4);
        for r in 0..4 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        Puzzle { board, clues }
    }

    #[test]
    fn solve_full_board_full_clues() {
        let puzzle = make_4x4_puzzle_full();
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], make_4x4_solution());
    }

    #[test]
    fn solve_empty_board_full_clues() {
        let sol = make_4x4_solution();
        let clues = Clues::from_solution(&sol);
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], sol);
    }

    #[test]
    fn solve_partial_clues_multiple_solutions() {
        let sol = make_4x4_solution();
        let mut clues = Clues::from_solution(&sol);
        for i in 0..4 {
            clues.set_top(i, None);
            clues.set_bottom(i, None);
            clues.set_left(i, None);
            clues.set_right(i, None);
        }
        clues.set_top(0, Some(3));
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 10);
        assert!(solutions.len() > 1, "should find multiple solutions");
        for s in &solutions {
            let derived = Clues::from_solution(s);
            assert_eq!(derived.top(0), Some(3));
        }
    }

    #[test]
    fn solve_limit_parameter() {
        let sol = make_4x4_solution();
        let mut clues = Clues::from_solution(&sol);
        for i in 0..4 {
            clues.set_top(i, None);
            clues.set_bottom(i, None);
            clues.set_left(i, None);
            clues.set_right(i, None);
        }
        let board = Board::new_empty(4);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 1);
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn solve_contradictory_input() {
        let mut board = Board::new_empty(4);
        board.set(0, 0, Some(1));
        board.set(0, 1, Some(1));
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 10);
        assert!(solutions.is_empty());
    }

    #[test]
    fn solve_partial_board_no_clues() {
        let sol = make_4x4_solution();
        let mut board = Board::new_empty(4);
        for r in 0..2 {
            for c in 0..4 {
                board.set(r, c, Some(sol.get(r, c)));
            }
        }
        let clues = Clues::new_all_none(4);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 10);
        assert!(!solutions.is_empty());
        for s in &solutions {
            for r in 0..2 {
                for c in 0..4 {
                    assert_eq!(s.get(r, c), sol.get(r, c));
                }
            }
        }
    }

    #[test]
    fn solve_n7_roundtrip() {
        let cells = vec![
            1, 2, 3, 4, 5, 6, 7, //
            2, 3, 4, 5, 6, 7, 1, //
            3, 4, 5, 6, 7, 1, 2, //
            4, 5, 6, 7, 1, 2, 3, //
            5, 6, 7, 1, 2, 3, 4, //
            6, 7, 1, 2, 3, 4, 5, //
            7, 1, 2, 3, 4, 5, 6, //
        ];
        let sol = Solution::new(7, cells);
        let clues = Clues::from_solution(&sol);
        let board = Board::new_empty(7);
        let puzzle = Puzzle { board, clues };
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 2);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], sol);
    }

    #[test]
    fn solve_limit_zero() {
        let puzzle = make_4x4_puzzle_full();
        let solver = SatSolver;
        let solutions = solver.solve(&puzzle, 0);
        assert!(solutions.is_empty());
    }

    #[test]
    fn count_visible_basic() {
        assert_eq!(count_visible(&[1, 2, 3, 4]), 4);
        assert_eq!(count_visible(&[4, 3, 2, 1]), 1);
        assert_eq!(count_visible(&[2, 1, 4, 3]), 2);
    }

    #[test]
    fn valid_permutations_count() {
        // n=4, clue=1 from front means first element must be 4
        let perms = valid_permutations(4, Some(1), None);
        assert!(perms.iter().all(|p| p[0] == 4));
        // n=4, clue=4 from front means ascending: [1,2,3,4]
        let perms = valid_permutations(4, Some(4), None);
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0], vec![1, 2, 3, 4]);
    }

    #[test]
    fn cross_validate_with_backtracking() {
        use crate::BacktrackingSolver;

        let test_cases: Vec<(Puzzle, Solution)> = vec![
            // 4x4: full clues, empty board
            {
                let sol = make_4x4_solution();
                let clues = Clues::from_solution(&sol);
                (
                    Puzzle {
                        board: Board::new_empty(4),
                        clues,
                    },
                    sol,
                )
            },
            // 5x5: sparse clues, empty board (generated puzzle)
            {
                let sol = Solution::new(
                    5,
                    vec![
                        3, 4, 2, 1, 5, //
                        2, 1, 3, 5, 4, //
                        1, 3, 5, 4, 2, //
                        5, 2, 4, 3, 1, //
                        4, 5, 1, 2, 3, //
                    ],
                );
                let mut clues = Clues::new_all_none(5);
                clues.set_top(2, Some(3));
                clues.set_bottom(1, Some(1));
                clues.set_bottom(4, Some(3));
                clues.set_left(1, Some(3));
                clues.set_right(2, Some(3));
                clues.set_right(3, Some(4));
                clues.set_right(4, Some(2));
                let mut board = Board::new_empty(5);
                board.set(4, 3, Some(2));
                (Puzzle { board, clues }, sol)
            },
            // 7x7: sparse clues, sparse board (generated puzzle)
            {
                let sol = Solution::new(
                    7,
                    vec![
                        2, 6, 5, 4, 1, 7, 3, //
                        4, 7, 1, 3, 5, 6, 2, //
                        1, 5, 4, 6, 3, 2, 7, //
                        3, 1, 7, 2, 6, 5, 4, //
                        6, 2, 3, 1, 7, 4, 5, //
                        7, 4, 6, 5, 2, 3, 1, //
                        5, 3, 2, 7, 4, 1, 6, //
                    ],
                );
                let mut clues = Clues::new_all_none(7);
                clues.set_top(0, Some(4));
                clues.set_top(2, Some(2));
                clues.set_top(3, Some(3));
                clues.set_top(5, Some(1));
                clues.set_top(6, Some(2));
                clues.set_bottom(1, Some(4));
                clues.set_bottom(4, Some(2));
                clues.set_bottom(5, Some(6));
                clues.set_bottom(6, Some(2));
                clues.set_left(2, Some(4));
                clues.set_right(3, Some(4));
                clues.set_right(5, Some(5));
                let mut board = Board::new_empty(7);
                board.set(1, 3, Some(3));
                board.set(1, 4, Some(5));
                board.set(2, 0, Some(1));
                board.set(4, 1, Some(2));
                (Puzzle { board, clues }, sol)
            },
        ];

        let bt = BacktrackingSolver;
        let sat = SatSolver;

        for (i, (puzzle, expected)) in test_cases.iter().enumerate() {
            let bt_solutions = bt.solve(puzzle, 2);
            let sat_solutions = sat.solve(puzzle, 2);
            assert_eq!(bt_solutions.len(), 1, "case {i}: bt should find unique solution");
            assert_eq!(sat_solutions.len(), 1, "case {i}: sat should find unique solution");
            assert_eq!(bt_solutions[0], *expected, "case {i}: bt solution mismatch");
            assert_eq!(sat_solutions[0], *expected, "case {i}: sat solution mismatch");
        }
    }
}
