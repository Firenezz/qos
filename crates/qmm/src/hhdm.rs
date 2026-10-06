//! Bootloader-provided HHDM (higher-half direct map) offset.
//!
//! Limine and similar bootloaders choose the direct-map virtual base at runtime.
//! Store that value once during early boot and use it for physical→virtual
//! translation instead of a hard-coded constant.

use core::sync::atomic::{AtomicUsize, Ordering};

static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Store the HHDM / direct-map offset provided by the bootloader.
#[inline]
pub fn set_hhdm_offset(offset: usize) {
    HHDM_OFFSET.store(offset, Ordering::Release);
}

/// Returns the bootloader-provided HHDM offset, if [`set_hhdm_offset`] was called
/// with a non-zero value.
#[inline]
pub fn try_hhdm_offset() -> Option<usize> {
    let offset = HHDM_OFFSET.load(Ordering::Acquire);
    if offset == 0 { None } else { Some(offset) }
}

/// Returns the HHDM offset, or `default` if none was registered.
#[inline]
pub fn hhdm_offset_or(default: usize) -> usize {
    try_hhdm_offset().unwrap_or(default)
}
