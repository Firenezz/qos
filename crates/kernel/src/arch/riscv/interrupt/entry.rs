//! Supervisor trap entry via the `riscv-interrupt-s` ABI.
//!
//! LLVM emits the register save/restore prologue/epilogue and `sret`. This path
//! does **not** swap onto the dedicated trap stack (`sscratch`); that stack is
//! still installed for later nested-trap work. Handlers run on the interrupted
//! `sp` (safe while `SIE` stays clear for the duration of the trap).

use super::dispatch;
use super::frame::TrapFrame;

/// Direct-mode `stvec` target. Compiler saves clobbered regs and returns with `sret`.
///
/// Do not log here: a fault inside the logger re-enters this path and recurses.
#[unsafe(no_mangle)]
pub extern "riscv-interrupt-s" fn trap_entry() {
    let mut frame = TrapFrame::capture();
    dispatch::handle(&mut frame);
    frame.restore_csrs();
}
