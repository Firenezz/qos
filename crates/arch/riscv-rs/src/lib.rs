#![no_std]
#![warn(missing_docs)]

//! A Rust library for RISC-V architecture support
//!
//! This crate provides safe abstractions for working with RISC-V architecture features,
//! including register access and instruction encoding/decoding.

#[cfg(not(feature = "std"))]
extern crate core as std;

use core::panic;

//use registers::general::*;

//pub mod registers;
//pub mod instructions;

// If on a target arch riscv then activate the riscv feature

/// Test function for assembly operations
///
/// This is a temporary test function that will be removed in future versions.
#[doc(hidden)]
pub fn test_asm() {
    //let _test = ra::read();
    //gp::write(0x1234);
    //fp::write(_test);

    //let _test = unsafe { macros::_read() };

    //panic!("test");
}