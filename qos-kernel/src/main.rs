#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[export_name = "kernel_main"]
extern fn main() -> ! {
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}

use core::panic::PanicInfo;

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    loop {}
}