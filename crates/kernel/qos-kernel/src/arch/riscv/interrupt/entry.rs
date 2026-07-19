//! Asm trap entry / exit and hand-off into the Rust dispatcher.
//!
//! - `tp` → [`HartLocal`](crate::arch::riscv::device::cpu::HartLocal) (trap stack
//!   base/top live there)
//! - `sscratch` mirrors the trap-stack top for the `csrrw` swap with `sp`
//!
//! Nested traps: interrupted `sp` already in `[base, top)`.

use super::dispatch;
use super::frame::TrapFrame;
use crate::arch::riscv::device::cpu::off;

/// Supervisor trap vector base (Direct mode). Defined in `global_asm!` below.
pub(super) use asm_sym::trap_entry;

mod asm_sym {
    unsafe extern "C" {
        pub fn trap_entry();
    }
}

core::arch::global_asm!(
    r#"
    .section .text.trap, "ax"
    .global trap_entry
    .align 2
trap_entry:
    # sp <-> sscratch: sp becomes trap-stack top, sscratch becomes interrupted sp.
    csrrw sp, sscratch, sp
    beqz sp, trap_double_fault
    beqz tp, trap_double_fault

    # sp must match HartLocal.trap_stack_top (tp + {top_off}).
    ld t0, {top_off}(tp)
    bne sp, t0, trap_double_fault

    # Interrupted sp (sscratch) must be outside [base, top).
    csrr t0, sscratch
    ld t1, {base_off}(tp)
    bltu t0, t1, 1f
    ld t1, {top_off}(tp)
    bgeu t0, t1, 1f
    j trap_double_fault
1:

    addi sp, sp, -288

    sd ra,    0(sp)
    csrr ra, sscratch
    sd ra,    8(sp)
    sd gp,   16(sp)
    sd tp,   24(sp)
    sd t0,   32(sp)
    sd t1,   40(sp)
    sd t2,   48(sp)
    sd s0,   56(sp)
    sd s1,   64(sp)
    sd a0,   72(sp)
    sd a1,   80(sp)
    sd a2,   88(sp)
    sd a3,   96(sp)
    sd a4,  104(sp)
    sd a5,  112(sp)
    sd a6,  120(sp)
    sd a7,  128(sp)
    sd s2,  136(sp)
    sd s3,  144(sp)
    sd s4,  152(sp)
    sd s5,  160(sp)
    sd s6,  168(sp)
    sd s7,  176(sp)
    sd s8,  184(sp)
    sd s9,  192(sp)
    sd s10, 200(sp)
    sd s11, 208(sp)
    sd t3,  216(sp)
    sd t4,  224(sp)
    sd t5,  232(sp)
    sd t6,  240(sp)

    csrr t0, sepc
    sd t0,  248(sp)
    csrr t0, sstatus
    sd t0,  256(sp)
    csrr t0, scause
    sd t0,  264(sp)
    csrr t0, stval
    sd t0,  272(sp)

    # Re-arm sscratch from HartLocal.trap_stack_top (tp still valid).
    ld t0, {top_off}(tp)
    csrw sscratch, t0

    mv a0, sp
    call {trap_handler}

    ld t0,  248(sp)
    csrw sepc, t0
    ld t0,  256(sp)
    csrw sstatus, t0

    ld ra,    0(sp)
    ld gp,   16(sp)
    ld tp,   24(sp)
    ld t0,   32(sp)
    ld t1,   40(sp)
    ld t2,   48(sp)
    ld s0,   56(sp)
    ld s1,   64(sp)
    ld a0,   72(sp)
    ld a1,   80(sp)
    ld a2,   88(sp)
    ld a3,   96(sp)
    ld a4,  104(sp)
    ld a5,  112(sp)
    ld a6,  120(sp)
    ld a7,  128(sp)
    ld s2,  136(sp)
    ld s3,  144(sp)
    ld s4,  152(sp)
    ld s5,  160(sp)
    ld s6,  168(sp)
    ld s7,  176(sp)
    ld s8,  184(sp)
    ld s9,  192(sp)
    ld s10, 200(sp)
    ld s11, 208(sp)
    ld t3,  216(sp)
    ld t4,  224(sp)
    ld t5,  232(sp)
    ld t6,  240(sp)

    ld sp,    8(sp)
    sret

trap_double_fault:
    j trap_double_fault
    "#,
    trap_handler = sym trap_handler,
    top_off = const off::TRAP_STACK_TOP,
    base_off = const off::TRAP_STACK_BASE,
);

/// Called from [`trap_entry`] with `a0` = pointer to the frame on the trap stack.
#[unsafe(no_mangle)]
extern "C" fn trap_handler(frame: &mut TrapFrame) {
    dispatch::handle(frame);
}
