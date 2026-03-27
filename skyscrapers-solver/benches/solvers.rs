#![feature(test)]

extern crate test;

use test::Bencher;

use skyscrapers_core::Puzzle;
use skyscrapers_solver::{BacktrackingSolver, SatSolver, Solver};

fn puzzle_4x4() -> Puzzle {
    "\
        3 . . .
      +---------+
    . | . . . . | .
    3 | . . . . | .
    . | . . . . | 1
    . | . . . . | .
      +---------+
        . 3 3 ."
        .parse()
        .unwrap()
}

fn puzzle_5x5() -> Puzzle {
    "\
        . . 3 . .
      +-----------+
    . | . . . . . | .
    3 | . . . . . | .
    . | . . . . . | 3
    . | . . . . . | 4
    . | . . . 2 . | 2
      +-----------+
        . 1 . . 3"
        .parse()
        .unwrap()
}

fn puzzle_7x7() -> Puzzle {
    "\
        4 . 2 3 . 1 2
      +---------------+
    . | . . . . . . . | .
    . | . . . 3 5 . . | .
    4 | 1 . . . . . . | .
    . | . . . . . . . | 4
    . | . 2 . . . . . | .
    . | . . . . . . . | 5
    . | . . . . . . . | .
      +---------------+
        . 4 . . 2 6 2"
        .parse()
        .unwrap()
}

fn puzzle_6x6() -> Puzzle {
    "\
        3 2 1 . 2 .
      +-------------+
    . | . . . . . . | .
    . | . . . . . . | .
    2 | . . . . 2 . | .
    5 | . . . . . . | .
    . | . . . 1 . . | 2
    . | . . . . . . | 4
      +-------------+
        . . 2 . . 2"
        .parse()
        .unwrap()
}

#[bench]
fn backtracking_4x4(b: &mut Bencher) {
    let puzzle = puzzle_4x4();
    b.iter(|| BacktrackingSolver.solve(&puzzle, 2));
}

#[bench]
fn sat_4x4(b: &mut Bencher) {
    let puzzle = puzzle_4x4();
    b.iter(|| SatSolver.solve(&puzzle, 2));
}

#[bench]
fn backtracking_5x5(b: &mut Bencher) {
    let puzzle = puzzle_5x5();
    b.iter(|| BacktrackingSolver.solve(&puzzle, 2));
}

#[bench]
fn sat_5x5(b: &mut Bencher) {
    let puzzle = puzzle_5x5();
    b.iter(|| SatSolver.solve(&puzzle, 2));
}

#[bench]
fn backtracking_6x6(b: &mut Bencher) {
    let puzzle = puzzle_6x6();
    b.iter(|| BacktrackingSolver.solve(&puzzle, 2));
}

#[bench]
fn sat_6x6(b: &mut Bencher) {
    let puzzle = puzzle_6x6();
    b.iter(|| SatSolver.solve(&puzzle, 2));
}

#[bench]
fn backtracking_7x7(b: &mut Bencher) {
    let puzzle = puzzle_7x7();
    b.iter(|| BacktrackingSolver.solve(&puzzle, 2));
}

#[bench]
fn sat_7x7(b: &mut Bencher) {
    let puzzle = puzzle_7x7();
    b.iter(|| SatSolver.solve(&puzzle, 2));
}
