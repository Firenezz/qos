//! Saved register / CSR state for a trap.

/// Trap frame laid out for the asm stub in [`super::entry`].
///
/// Size is 288 bytes (16-byte aligned) so `sp` stays ABI-aligned across the call
/// into Rust.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapFrame {
    pub ra: u64,
    /// SP before this frame was pushed.
    pub sp: u64,
    pub gp: u64,
    pub tp: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub s0: u64,
    pub s1: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,
    pub sepc: u64,
    pub sstatus: u64,
    pub scause: u64,
    pub stval: u64,
    pub _pad: u64,
}

const _: () = assert!(core::mem::size_of::<TrapFrame>() == 288);
const _: () = assert!(core::mem::align_of::<TrapFrame>() == 8);
