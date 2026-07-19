//! RISC-V supervisor trap / interrupt handling.
//!
//! Direct-mode `stvec` → asm stub. Trap stack bounds live in [`HartLocal`]
//! (`tp`); `sscratch` mirrors the top for `csrrw` with `sp`.
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

/// Install `tp` → hart-local, trap stack, and the trap vector.
///
/// Leaves supervisor IRQs masked (`sie = 0`, `sstatus.SIE = 0`).
///
/// # Safety
///
/// Call once on the BSP (and later on each hart with its own `HartLocal`) before
/// relying on traps. Nested traps spin in `trap_double_fault`.
pub unsafe fn init() {
    // SAFETY: BSP only; sets `tp`, `sscratch`, and stack bounds.
    unsafe {
        stack::install();
    }

    let mut vec = stvec::Local::from_bits(0);
    vec.set_mode(StvecMode::Direct);
    vec.set_base(trap_entry as *const () as usize as u64);
    stvec::write(vec);

    sie::write_raw(0);
    sstatus::clear(sstatus::Flags::SIE);
}

/// Enable supervisor interrupts (`sstatus.SIE = 1`).
#[inline(always)]
pub fn enable() {
    sstatus::set(sstatus::Flags::SIE);
}

/// Disable supervisor interrupts (`sstatus.SIE = 0`).
#[inline(always)]
pub fn disable() {
    sstatus::clear(sstatus::Flags::SIE);
}

/// Returns whether supervisor interrupts are enabled.
#[inline(always)]
pub fn is_enabled() -> bool {
    sstatus::read().sie()
}
