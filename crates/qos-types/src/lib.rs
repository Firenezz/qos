#![no_std]

//! Portable shared types for qos.
//!
//! Architecture-specific code (CSRs, registers, asm) lives in arch crates such as
//! `riscv-rs`, not here.
//!
//! Host tests: `cargo test -p qos-types --features std`.

pub mod memory;
pub mod paging;

pub use memory::{
    align_down, align_up, PhysAddr, PhysAddrNotValid, VirtAddr, VirtAddrNotValid,
    ADDRESS_SPACE_SIZE, PHYS_ADDR_BITS,
};
