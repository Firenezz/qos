//! Architecture abstraction.
//!
//! Boot reaches [`crate::boot::kernel_main`] with a [`crate::boot::BootInfo`].
//! Arch code owns CPU identity, early bring-up, and halt — not bootloader parsing.

pub mod riscv;

/// Per-CPU operations for the target ISA.
pub trait Cpu {
    /// Opaque hardware CPU / hart identifier.
    type Id: Copy + core::fmt::Debug + Eq;

    /// Read this CPU's id (e.g. `mhartid` / `tp`-backed hart id).
    fn id() -> Self::Id;

    /// Wait for an interrupt (`wfi` / `hlt` / …).
    fn wait_for_interrupt();

    /// Park this CPU forever (interrupts off if applicable).
    fn halt() -> !;
}

/// Target architecture façade used by the rest of the kernel.
pub trait Arch: Sized {
    type Cpu: Cpu;

    /// Very early arch setup after GP/stack are valid, before protocol handoff returns
    /// into [`crate::boot::kernel_main`]. Safe to leave empty.
    ///
    /// # Safety
    /// Called once on the BSP with interrupt state undefined.
    unsafe fn early_init() {}

    fn init_logging() {}

    #[inline(always)]
    fn cpu_id() -> <Self::Cpu as Cpu>::Id {
        Self::Cpu::id()
    }

    #[inline(always)]
    fn halt() -> ! {
        Self::Cpu::halt()
    }
}

/// The architecture this kernel binary is built for.
#[cfg(target_arch = "riscv64")]
pub type Current = riscv::Riscv64;
