//! Process-global hooks for `skyscrapers-analysis`. Gated behind the
//! `analysis-hooks` cargo feature so production builds (CLI, wasm) compile
//! these out entirely. The end-user `LogicSolver` API is unchanged either way.
//!
//! Limitation: `CluePruning` runs once during `SolveState::new` and is not
//! routed through the dispatch loop, so disabling it via this module has no
//! effect. All other techniques in `Technique` are honored.

use super::difficulty::Technique;

#[cfg(feature = "analysis-hooks")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "analysis-hooks")]
static DISABLED_MASK: AtomicU64 = AtomicU64::new(0);

fn technique_bit(t: Technique) -> u64 {
    1u64 << (t as u8)
}

/// Disable the given techniques in the dispatch loop process-wide.
/// Replaces any previous selection.
#[cfg(feature = "analysis-hooks")]
pub fn set_disabled(techniques: &[Technique]) {
    let mask = techniques
        .iter()
        .copied()
        .fold(0u64, |m, t| m | technique_bit(t));
    DISABLED_MASK.store(mask, Ordering::Relaxed);
}

/// Clear all disabled techniques.
#[cfg(feature = "analysis-hooks")]
pub fn clear_disabled() {
    DISABLED_MASK.store(0, Ordering::Relaxed);
}

#[inline]
pub(crate) fn is_disabled(t: Technique) -> bool {
    #[cfg(feature = "analysis-hooks")]
    {
        DISABLED_MASK.load(Ordering::Relaxed) & technique_bit(t) != 0
    }
    #[cfg(not(feature = "analysis-hooks"))]
    {
        let _ = technique_bit(t);
        false
    }
}
