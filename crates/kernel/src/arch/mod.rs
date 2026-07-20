//! Architecture abstraction.
//!
//! Boot reaches [`crate::boot::kernel_main`] with a [`crate::boot::BootInfo`].
//! Arch code owns CPU identity, early bring-up, and halt — not bootloader parsing.

pub mod riscv;
pub mod device;

/// Per-CPU operations for the target ISA.
pub trait Processor {
    /// Wait for an interrupt (`wfi` / `hlt` / …).
    fn wait_for_interrupt();

    /// Park this CPU forever (interrupts off if applicable).
    fn halt() -> !;
}

/// Target architecture façade used by the rest of the kernel.
pub trait Arch: Sized {
    type Cpu: Processor;

    /// Very early arch setup after GP/stack are valid, before protocol handoff returns
    /// into [`crate::boot::kernel_main`]. Safe to leave empty.
    ///
    /// # Safety
    /// Called once on the BSP with interrupt state undefined.
    unsafe fn early_init() {}

    /// Initialize the architecture specific state for the boot information.
    fn init(info: &crate::boot::BootInfo);

    /// Initialize the architecture specific state for each processor.
    /// 
    /// This function is called for each processor after the APs entered the kernel main function.
    fn init_per_processor();

    /// Initialize the logging system.
    /// 
    /// # Safety
    /// 
    /// Called once by the BSP only
    fn init_logging();

    /// Enable IRQs on this CPU.
    fn irq_enable();

    /// Disable IRQs on this CPU.
    fn irq_disable();

    /// Whether IRQs are enabled on this CPU.
    fn irq_is_enabled() -> bool;

    #[inline(always)]
    fn halt() -> ! {
        Self::Cpu::halt()
    }

    fn with_disabled_irqs<F: FnOnce() -> R, R>(f: F) -> R {
        let were_enabled = Self::irq_is_enabled();
        Self::irq_disable();
        let result = f();
        if were_enabled {
            Self::irq_enable();
        }
        result
    }
}

/// The architecture this kernel binary is built for.
#[cfg(target_arch = "riscv64")]
pub type Current = riscv::Riscv64;

pub fn enable_irq() {
    Current::irq_enable();
}

pub fn disable_irq() {
    Current::irq_disable();
}

pub fn irq_is_enabled() -> bool {
    Current::irq_is_enabled()
}

pub fn with_disabled_irqs<F: FnOnce() -> R, R>(f: F) -> R {
    Current::with_disabled_irqs(f)
}

pub fn init_logging() {
    Current::init_logging();
}
