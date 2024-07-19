use core::arch::asm;
use core::sync::atomic::{compiler_fence, Ordering};

/// Returns whether interrupts are enabled.
#[inline]
pub fn are_enabled() -> bool {
    false
}
