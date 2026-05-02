//! Per-thread hooks for `skyscrapers-analysis`. Gated behind the
//! `analysis-hooks` cargo feature so production builds (CLI, wasm) compile
//! these out entirely. The end-user `LogicSolver` API is unchanged either way.
//!
//! State is held in thread-local storage. The set-disabled → solve →
//! clear-disabled sequence is therefore confined to the thread that performs
//! it, so concurrent analysis runs (or parallel `cargo test` threads) cannot
//! observe each other's masks.
//!
//! Limitation: `CluePruning` runs once during `SolveState::new` and is not
//! routed through the dispatch loop, so disabling it via this module has no
//! effect. All other techniques in `Technique` are honored.

use super::difficulty::Technique;

#[cfg(feature = "analysis-hooks")]
use std::cell::Cell;

#[cfg(feature = "analysis-hooks")]
thread_local! {
    static DISABLED_MASK: Cell<u64> = const { Cell::new(0) };
}

fn technique_bit(t: Technique) -> u64 {
    let shift = t as u32;
    debug_assert!(
        shift < u64::BITS,
        "Technique discriminant {shift} does not fit in the u64 analysis-hook bitmask"
    );
    1u64.checked_shl(shift).unwrap_or(0)
}

/// Disable the given techniques in the dispatch loop on the current thread.
/// Replaces any previous selection on this thread.
#[cfg(feature = "analysis-hooks")]
pub fn set_disabled(techniques: &[Technique]) {
    let mask = techniques
        .iter()
        .copied()
        .fold(0u64, |m, t| m | technique_bit(t));
    DISABLED_MASK.with(|cell| cell.set(mask));
}

/// Clear all disabled techniques on the current thread.
#[cfg(feature = "analysis-hooks")]
pub fn clear_disabled() {
    DISABLED_MASK.with(|cell| cell.set(0));
}

#[inline]
pub(crate) fn is_disabled(t: Technique) -> bool {
    #[cfg(feature = "analysis-hooks")]
    {
        DISABLED_MASK.with(|cell| cell.get() & technique_bit(t) != 0)
    }
    #[cfg(not(feature = "analysis-hooks"))]
    {
        let _ = technique_bit(t);
        false
    }
}
