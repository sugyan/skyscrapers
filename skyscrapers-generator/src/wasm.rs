use std::str::FromStr;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{GeneratorParams, generate};
use skyscrapers_core::{Board, Puzzle, Solution};
use skyscrapers_solver::{Difficulty, LogicSolver, logic::difficulty::Step};

#[derive(Serialize)]
struct PuzzleResult {
    puzzle: Puzzle,
    solution: Solution,
    /// The difficulty the logic solver rated the generated puzzle at.
    /// `None` when the puzzle is harder than the logic solver can rate.
    difficulty: Option<Difficulty>,
}

/// Generate a Skyscrapers puzzle of size `n` with the given `seed`.
///
/// When `difficulty` is provided (one of "easy", "medium", "hard", "expert",
/// "master"), the generator retries until the produced puzzle exactly matches
/// that target difficulty.
///
/// Returns a JS object with `puzzle`, `solution`, and `difficulty` fields.
/// `difficulty` is the level the logic solver rated the generated puzzle at,
/// omitted (`undefined`) when the puzzle is harder than the logic solver can
/// rate (only possible when no target `difficulty` was requested).
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
    let (puzzle, solution, difficulty) =
        generate(&mut rng, &params).map_err(|e| JsError::new(&e.to_string()))?;

    let result = PuzzleResult {
        puzzle,
        solution,
        difficulty,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

#[derive(Serialize)]
struct HintResult {
    step: Step,
    candidates: Vec<Vec<Vec<u8>>>,
}

/// Compute the next logical hint for the current board state.
///
/// `puzzle` carries the original clues; `board` is the user's current
/// confirmed values (givens + user-placed digits). `user_candidates` is an
/// optional `n × n` grid of the values the user has currently pencilled
/// in — when provided, hints whose every elimination is already reflected
/// by the user's pencil marks are skipped so the same hint isn't returned
/// repeatedly. The function returns `null` when no further logical step
/// is available.
#[wasm_bindgen]
pub fn next_hint(
    puzzle: JsValue,
    board: JsValue,
    user_candidates: JsValue,
) -> Result<JsValue, JsError> {
    let puzzle: Puzzle =
        serde_wasm_bindgen::from_value(puzzle).map_err(|e| JsError::new(&e.to_string()))?;
    let board: Board =
        serde_wasm_bindgen::from_value(board).map_err(|e| JsError::new(&e.to_string()))?;
    let user_candidates: Option<Vec<Vec<Vec<u8>>>> =
        if user_candidates.is_undefined() || user_candidates.is_null() {
            None
        } else {
            Some(
                serde_wasm_bindgen::from_value(user_candidates)
                    .map_err(|e| JsError::new(&e.to_string()))?,
            )
        };

    match LogicSolver.next_step_with_candidates(&puzzle, &board, user_candidates.as_deref()) {
        Some((step, candidates)) => {
            let result = HintResult { step, candidates };
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
        }
        None => Ok(JsValue::NULL),
    }
}
