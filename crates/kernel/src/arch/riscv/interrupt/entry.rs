//! Supervisor trap entry (naked, direct-mode `stvec`).
//!
//! Armed: `sscratch` holds `&HartLocal`. On entry we `csrrw` with `tp` so the
//! handler runs with `tp` = percpu and `sscratch` = parked TLS. Until armed,
//! both `tp` and `sscratch` are 0, so the early path needs no undo.

use core::arch::naked_asm;

use super::dispatch;
use super::frame::TrapFrame;

/// Direct-mode `stvec` target. Must stay 4-byte aligned.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[rustc_align(4)]
pub unsafe extern "custom" fn trap_handler() {
    // `csrrw tp, sscratch, tp`: tp := percpu (or 0), sscratch := TLS.
    naked_asm!(
        concat!(
            "csrrw tp, sscratch, tp\n",
            "beqz  tp, 3f\n",

            // Armed: tp = &HartLocal, sscratch = TLS.
            "sd    t0, 0(tp)\n",
            "csrr  t2, sstatus\n",
            "andi  t2, t2, 1 << 8\n",
            "bnez  t2, 2f\n",

            // U-mode → trap stack (top at HartLocal + 8).
            "ld    t0, 0(tp)\n",
            "sd    sp, 0(tp)\n",
            "ld    sp, 8(tp)\n",
            crate::push_registers!(),
            "mv    a0, sp\n",
            "jal   {handle}\n",
            "j     4f\n",

            // S-mode (armed): same stack.
            "2:\n",
            "ld    t0, 0(tp)\n",

            // Early (tp == sscratch == 0) and armed S-mode join here.
            "3:\n",
            "addi  sp, sp, -2 * 8\n",
            crate::push_registers!(),
            "addi  t1, sp, 34 * 8\n",
            "sd    t1, (1 * 8)(sp)\n",
            "sd    fp, (32 * 8)(sp)\n",
            "addi  fp, sp, 34 * 8\n",
            "mv    a0, sp\n",
            "jal   {handle}\n",

            // Armed: sscratch still holds TLS — swap back before pop.
            // Early: both still 0; the beqz skips the swap.
            "4:\n",
            "csrr  t0, sscratch\n",
            "beqz  t0, 6f\n",
            "csrrw tp, sscratch, tp\n",
            "6:\n",
            crate::pop_registers!(),
            "sret\n",
        ),
        handle = sym dispatch::handle,
    )
}