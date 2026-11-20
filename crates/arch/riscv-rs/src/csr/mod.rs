//! RISC-V control and status registers (CSRs).
//!
//! Provides typed local read-modify-write transactions and one-shot
//! `csrrs` / `csrrc` helpers for supervisor-mode CSRs (RV64).

#[macro_use]
mod macros;
mod local;

pub use local::Local;

pub mod satp;
pub mod scause;
pub mod sepc;
pub mod sie;
pub mod sip;
pub mod sscratch;
pub mod sstatus;
pub mod stval;
pub mod stvec;

/// Marker trait for a CSR at a fixed privileged-ISA address.
pub trait Csr {
    /// 12-bit CSR address.
    const ADDR: u16;
    /// Raw storage type (`u64` for RV64).
    type Bits: Copy + Eq;
}

/// CSRs that can be read with `csrr`.
pub trait Readable: Csr {
    /// Read the raw CSR value.
    fn read_raw() -> Self::Bits;
}

/// CSRs that can be written with `csrw` / `csrrs` / `csrrc`.
pub trait Writeable: Csr {
    /// Write the full raw CSR value (`csrw`).
    fn write_raw(bits: Self::Bits);
    /// Set bits (`csrrs`).
    fn set_bits(mask: Self::Bits);
    /// Clear bits (`csrrc`).
    fn clear_bits(mask: Self::Bits);
}
