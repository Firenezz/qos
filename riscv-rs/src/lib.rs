#![no_std]

use core::panic;

use registers::general::*;

pub mod instructions;
pub mod registers;

// If on a target arch riscv then activate the riscv feature

pub fn test_asm() {
    let _test = ra::read();
    gp::write(0x1234);
    fp::write(_test);

    //let _test = unsafe { macros::_read() };

    panic!("test");
}
