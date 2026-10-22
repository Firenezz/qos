//! Architecture abstraction.
//!
//! Boot reaches [`crate::kernel::kernel_main`] with a [`crate::boot::BootInfo`].
//! Arch code owns CPU identity, early bring-up, and halt — not bootloader parsing.

pub mod device;
pub mod riscv;
pub mod traits;

pub use traits::*;

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

pub fn init_logging(uart_address: Option<core::num::NonZeroU64>) {
    Current::init_logging(uart_address);
}

/// Register devices with the platform.
pub fn register_devices() {
    Current::register_devices();
}
