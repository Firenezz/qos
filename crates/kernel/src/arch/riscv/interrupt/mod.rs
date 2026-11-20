//! RISC-V supervisor trap / interrupt handling.
//!
//! Direct-mode `stvec` → [`trap_handler`](`entry::trap_handler`) (naked, 4-byte aligned).
//! A dedicated trap stack is installed into [`HartLocal`] for later nested-trap
//! use. `sscratch` stays 0 until the hart is armed; once armed, entry parks TLS
//! in `sscratch` and uses `tp` for percpu for the duration of the handler.
//!
//! [`HartLocal`]: crate::arch::riscv::device::cpu::HartLocal

#[macro_use]
mod macros;
mod cause;
mod dispatch;
mod entry;
mod frame;
mod stack;
mod exception;

pub use cause::{Exception, Interrupt};
pub use frame::TrapFrame;

use riscv_rs::csr::stvec::StvecMode;
use riscv_rs::csr::{sie, sstatus, stvec};

use entry::trap_handler;

/// Install the trap stack into [`HartLocal`] and the trap vector.
///
/// Leaves `sscratch` at 0 (early trap path) and supervisor IRQs masked
/// (`sie = 0`, `sstatus.SIE = 0`).
///
/// # Safety
///
/// Call once on the BSP (and later on each hart with its own `HartLocal`) before
/// relying on traps.
pub unsafe fn init() {
    // SAFETY: BSP only; fills hart-local stack bounds (does not arm `sscratch`).
    log::info!("Initializing interrupt handling");
    log::trace!("Installing trap stack");
    unsafe {
        stack::install();
    }

    log::trace!("Setting up trap vector");
    let mut vec = stvec::Local::from_bits(0);
    vec.set_mode(StvecMode::Direct);
    vec.set_base(trap_handler as *const () as usize as u64);
    stvec::write(vec);

    log::trace!("Masking supervisor interrupts");
    sie::write_raw(0);
    sstatus::clear(sstatus::Flags::SIE);
}

/// Enable supervisor interrupts (`sstatus.SIE = 1`).
///
/// Must not log: [`crate::console`] calls this via `with_disabled_irqs`.
#[inline(always)]
pub fn enable() {
    sstatus::set(sstatus::Flags::SIE);
}

/// Disable supervisor interrupts (`sstatus.SIE = 0`).
///
/// Must not log: [`crate::console`] calls this via `with_disabled_irqs`.
#[inline(always)]
pub fn disable() {
    sstatus::clear(sstatus::Flags::SIE);
}

/// Returns whether supervisor interrupts are enabled.
#[inline(always)]
pub fn is_enabled() -> bool {
    sstatus::read().sie()
}
