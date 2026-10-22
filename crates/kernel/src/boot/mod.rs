//! Boot protocols → [`BootInfo`] → [`crate::kernel::kernel_main`].
//!
//! Sequence (all bootloaders):
//! 1. `boot_main` (arch) — set GP, jump to protocol handoff
//! 2. `protocol::handoff` — build [`BootInfo`]
//! 3. [`crate::kernel::kernel_main`] — shared kernel

pub mod capabilities;
pub mod info;
pub mod memory;

#[cfg(feature = "boot-limine")]
pub mod limine;

pub use capabilities::Capability;
pub use info::{Architecture, BootInfo, CpuInfo, CpuMap, FramebufferInfo};
pub use memory::{
    BootMemoryKind, BootMemoryMap, BootMemoryRegion, BootMemorySnapshot, ReclaimStage,
};

pub trait BootLoader {
    fn handoff(&self) -> !;

    /// Bootstrap the CPU.
    ///
    /// This function request the bootloader specific context to start the processor with a thread pointer as extra argument.
    ///
    /// the bootloader must then send the processor back to the kernel at
    fn bootstrap(&self) -> Result<(), &'static str>;
}
