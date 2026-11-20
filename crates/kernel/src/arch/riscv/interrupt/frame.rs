//! Trap CSR snapshot visible to Rust.
//!
//! GPRs are saved/restored by the naked entry (`push_registers!` /
//! `pop_registers!`) and are not exposed here.

use riscv_rs::csr::{scause, sepc, sstatus, stval};

/// CSR snapshot for the current trap.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapFrame {
    pub sepc: u64,
    pub sstatus: u64,
    pub scause: u64,
    pub stval: u64,
}

impl TrapFrame {
    /// Read trap CSRs at handler entry.
    #[inline]
    pub fn capture() -> Self {
        Self {
            sepc: sepc::read_raw(),
            sstatus: sstatus::read_raw(),
            scause: scause::read_raw(),
            stval: stval::read_raw(),
        }
    }

    /// Write back CSRs the handler may have changed (e.g. advanced `sepc`).
    #[inline]
    pub fn restore_csrs(self) {
        sepc::write_raw(self.sepc);
        sstatus::write_raw(self.sstatus);
    }
}
