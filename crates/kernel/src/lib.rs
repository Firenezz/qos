#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_riscv_interrupt)]
#![test_runner(crate::test_runner)]

extern crate alloc;

pub mod arch;
pub mod boot;
pub mod console;
pub mod definitions;
pub mod kernel;
pub mod helpers;

use crate::arch::Current;

pub fn halt() -> ! {
    log::info!("Halting...");
    <<Current as arch::Arch>::Cpu as crate::arch::Processor>::halt()
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    halt()
}

pub fn test_runner(_tests: &[&dyn Fn()]) {
    unimplemented!();
}
