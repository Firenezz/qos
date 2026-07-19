#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

pub mod arch;
pub mod boot;
pub mod definitions;
pub mod kernel;

use crate::arch::Current;

pub fn halt() -> ! {
    <<Current as arch::Arch>::Cpu as crate::arch::Cpu>::halt()
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    halt()
}

pub fn test_runner(_tests: &[&dyn Fn()]) {
    unimplemented!();
}
