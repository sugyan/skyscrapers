use std::str::FromStr;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{GeneratorParams, generate};
use skyscrapers_core::{Puzzle, Solution};
use skyscrapers_solver::Difficulty;

#[derive(Serialize)]
struct PuzzleResult {
    puzzle: Puzzle,
    solution: Solution,
}

/// Generate a Skyscrapers puzzle of size `n` with the given `seed`.
///
/// When `difficulty` is provided (one of "easy", "medium", "hard", "expert",
/// "master", "grandmaster"), the generator retries until the produced puzzle
/// exactly matches that target difficulty.
///
/// Returns a JS object with `puzzle` and `solution` fields.
#[wasm_bindgen]
pub fn generate_puzzle(n: u8, seed: u64, difficulty: Option<String>) -> Result<JsValue, JsError> {
    if !(1..=9).contains(&n) {
        return Err(JsError::new("n must be in range 1..=9"));
    }
    let n = n as usize;

    let parsed_difficulty = match difficulty {
        Some(s) => Some(
            Difficulty::from_str(&s)
                .map_err(|_| JsError::new(&format!("unknown difficulty: {s}")))?,
        ),
        None => None,
    };

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut params = GeneratorParams::new(n);
    if let Some(d) = parsed_difficulty {
        params = params.with_target_difficulty(d);
    }
    let (puzzle, solution) =
        generate(&mut rng, &params).map_err(|e| JsError::new(&e.to_string()))?;

    let result = PuzzleResult { puzzle, solution };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}
