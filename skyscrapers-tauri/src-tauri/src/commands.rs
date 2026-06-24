use std::str::FromStr;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

use skyscrapers_core::{Board, Puzzle, Solution};
use skyscrapers_generator::{GeneratorParams, generate};
use skyscrapers_solver::{Difficulty, LogicSolver, logic::difficulty::Step};

#[derive(Serialize)]
pub struct PuzzleResult {
    puzzle: Puzzle,
    solution: Solution,
    /// The difficulty the logic solver rated the generated puzzle at.
    /// `None` when the puzzle is harder than the logic solver can rate.
    difficulty: Option<Difficulty>,
}

#[derive(Serialize)]
pub struct HintResult {
    step: Step,
    candidates: Vec<Vec<Vec<u8>>>,
}

/// Generate a Skyscrapers puzzle. Mirrors the WASM `generate_puzzle` shape so
/// the TS-side TauriEngine can reuse WasmEngine's result-conversion logic.
///
/// `seed` is a decimal string (JavaScript BigInt → string) parsed to `u64`.
#[tauri::command]
pub fn generate_puzzle(
    n: u8,
    seed: String,
    difficulty: Option<String>,
) -> Result<PuzzleResult, String> {
    if !(1..=9).contains(&n) {
        return Err(format!("n must be in range 1..=9 (got {n})"));
    }
    let n = n as usize;

    let seed = u64::from_str(&seed).map_err(|e| format!("invalid seed: {e}"))?;

    let parsed_difficulty = match difficulty {
        Some(s) => Some(Difficulty::from_str(&s).map_err(|_| format!("unknown difficulty: {s}"))?),
        None => None,
    };

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut params = GeneratorParams::new(n);
    if let Some(d) = parsed_difficulty {
        params = params.with_target_difficulty(d);
    }

    // Distinct from the `difficulty` parameter above (the requested target):
    // this is the difficulty the solver rated the produced puzzle at.
    let (puzzle, solution, rated_difficulty) =
        generate(&mut rng, &params).map_err(|e| e.to_string())?;

    Ok(PuzzleResult {
        puzzle,
        solution,
        difficulty: rated_difficulty,
    })
}

#[tauri::command]
pub fn next_hint(
    puzzle: Puzzle,
    board: Board,
    user_candidates: Option<Vec<Vec<Vec<u8>>>>,
) -> Result<Option<HintResult>, String> {
    match LogicSolver.next_step_with_candidates(&puzzle, &board, user_candidates.as_deref()) {
        Some((step, candidates)) => Ok(Some(HintResult { step, candidates })),
        None => Ok(None),
    }
}
