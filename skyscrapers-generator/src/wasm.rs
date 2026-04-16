use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{GeneratorParams, generate};
use skyscrapers_core::{Puzzle, Solution};

#[derive(Serialize)]
struct PuzzleResult {
    puzzle: Puzzle,
    solution: Solution,
}

/// Generate a Skyscrapers puzzle of size `n` with the given `seed`.
///
/// Returns a JS object with `puzzle` and `solution` fields.
#[wasm_bindgen]
pub fn generate_puzzle(n: u8, seed: u64) -> Result<JsValue, JsError> {
    if !(1..=9).contains(&n) {
        return Err(JsError::new("n must be in range 1..=9"));
    }
    let n = n as usize;

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let params = GeneratorParams::new(n);
    let (puzzle, solution) =
        generate(&mut rng, &params).map_err(|e| JsError::new(&e.to_string()))?;

    let result = PuzzleResult { puzzle, solution };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}
