//! Boot protocols → [`BootInfo`] → [`kernel_main`].
//!
//! Sequence (all bootloaders):
//! 1. `boot_main` (arch) — set GP
//! 2. [`early_start`] — pick protocol adapter
//! 3. `protocol::handoff` — build [`BootInfo`]
//! 4. [`kernel_main`] — shared kernel

pub mod capabilities;
pub mod info;

#[cfg(feature = "boot-limine")]
pub mod limine;

pub use capabilities::Capability;
pub use info::{
    Architecture, BootInfo, CpuInfo, CpuMap, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion,
};