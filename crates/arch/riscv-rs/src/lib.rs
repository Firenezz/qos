#![no_std]
#![warn(missing_docs)]
#![allow(unused_attributes)]
#![allow(unused)]

//! RISC-V architecture support for qos (RV64gc).
//!
//! This crate currently exposes typed supervisor-mode CSR access. Field-packing
//! tests are host-only (`cargo test -p riscv-rs --features std`); CSR asm
//! requires `target_arch = "riscv64"`.

pub mod csr;
pub mod ecall;
pub mod interrupt;
pub mod operations;
