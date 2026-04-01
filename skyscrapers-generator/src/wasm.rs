use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{GeneratorParams, generate};
use skyscrapers_solver::{BacktrackingSolver, Solver};

#[derive(Serialize)]
struct PuzzleResult {
    n: usize,
    clues: CluesResult,
    board: Vec<Vec<Option<u8>>>,
    solution: Vec<Vec<u8>>,
}

#[derive(Serialize)]
struct CluesResult {
    top: Vec<Option<u8>>,
    bottom: Vec<Option<u8>>,
    left: Vec<Option<u8>>,
    right: Vec<Option<u8>>,
}

/// Generate a Skyscrapers puzzle of size `n` with the given `seed`.
///
/// Returns a JS object with `n`, `clues`, `board`, and `solution` fields.
#[wasm_bindgen]
pub fn generate_puzzle(n: u8, seed: u64) -> Result<JsValue, JsError> {
    if !(1..=9).contains(&n) {
        return Err(JsError::new("n must be in range 1..=9"));
    }
    let n = n as usize;

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let params = GeneratorParams::new(n, BacktrackingSolver);
    let puzzle = generate(&mut rng, &params);

    // Recover the solution by solving the generated puzzle
    let solutions = BacktrackingSolver.solve(&puzzle, 1);
    let solution = solutions
        .first()
        .ok_or_else(|| JsError::new("solver failed to find a solution for the generated puzzle"))?;

    let result = PuzzleResult {
        n,
        clues: CluesResult {
            top: (0..n).map(|i| puzzle.clues.top(i)).collect(),
            bottom: (0..n).map(|i| puzzle.clues.bottom(i)).collect(),
            left: (0..n).map(|i| puzzle.clues.left(i)).collect(),
            right: (0..n).map(|i| puzzle.clues.right(i)).collect(),
        },
        board: (0..n)
            .map(|r| (0..n).map(|c| puzzle.board.get(r, c)).collect())
            .collect(),
        solution: (0..n)
            .map(|r| (0..n).map(|c| solution.get(r, c)).collect())
            .collect(),
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}
