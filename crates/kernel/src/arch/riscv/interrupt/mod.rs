//! RISC-V supervisor trap / interrupt handling.
//!
//! Direct-mode `stvec` → [`trap_entry`](`entry::trap_entry`) (`riscv-interrupt-s`).
//! A dedicated trap stack is installed into [`HartLocal`] / `sscratch` for later
//! nested-trap use; the interrupt ABI entry currently runs on the interrupted stack.
//!
//! [`HartLocal`]: crate::arch::riscv::device::cpu::HartLocal

mod cause;
mod dispatch;
mod entry;
mod frame;
mod stack;

pub use cause::{Exception, Interrupt};
pub use frame::TrapFrame;

use riscv_rs::csr::{sie, sstatus, stvec};
use riscv_rs::csr::stvec::StvecMode;

use entry::trap_entry;

type TrapHandler = fn(frame: TrapFrame) -> !;

/// Install `tp` → hart-local, trap stack, and the trap vector.
///
/// Leaves supervisor IRQs masked (`sie = 0`, `sstatus.SIE = 0`).
///
/// # Safety
///
/// Call once on the BSP (and later on each hart with its own `HartLocal`) before
/// relying on traps.
pub unsafe fn init() {
    // SAFETY: BSP only; sets `tp`, `sscratch`, and stack bounds.
    log::info!("Initializing interrupt handling");
    log::trace!("Installing trap stack");
    unsafe {
        stack::install();
    }

    log::trace!("Setting up trap vector");
    let mut vec = stvec::Local::from_bits(0);
    vec.set_mode(StvecMode::Direct);
    vec.set_base(trap_entry as *const () as usize as u64);
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
