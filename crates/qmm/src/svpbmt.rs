//! Runtime SVPBMT (supervisor page-based memory types) availability.
//!
//! PBMT PTE bits must remain zero unless the hart implements the SVPBMT extension.

use core::sync::atomic::{AtomicBool, Ordering};

static SVPBMT_ENABLED: AtomicBool = AtomicBool::new(false);

/// Record whether the current hart supports SVPBMT.
#[inline]
pub fn set_svpbmt_enabled(enabled: bool) {
    SVPBMT_ENABLED.store(enabled, Ordering::Release);
}

/// Returns whether SVPBMT was reported available at boot.
#[inline]
pub fn svpbmt_enabled() -> bool {
    SVPBMT_ENABLED.load(Ordering::Acquire)
}
