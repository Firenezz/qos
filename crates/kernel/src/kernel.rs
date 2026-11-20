//! Shared kernel state and bootloader-agnostic entry.
//!
//! Subsystems own long-lived policy/state; [`entry`] owns bring-up after [`crate::boot`].

pub mod alloc;
pub mod cpu;
pub mod entry;
pub mod framebuffer;
pub mod memory;
pub mod os;

pub use entry::{kernel_main, kernel_secondary_main, pre_bootloader_init};
