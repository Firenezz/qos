#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

use core::{arch::asm, cell::UnsafeCell};

use common_boot::BootInfo;
use uefi::table::{Runtime, SystemTable};

static SYSTEM_TABLE: RacyCell<Option<SystemTable<Runtime>>> = RacyCell::new(None);

struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    const fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

impl<T> core::ops::Deref for RacyCell<T> {
    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    riscv_rs::test_asm();

    hcf()
}

//#[cfg(not(test))]
use core::panic::PanicInfo;

#[allow(dead_code)]
#[inline(never)]
/// Halt and Catch Fire
///
/// This function will disable interrupts and then wait for an interrupt to occur.
/// Effectively, stopping the cpu from doing anything else.
pub fn hcf() -> ! {
    // Halt and Catch Fire
    unsafe {
        #[cfg(feature = "hcf")]
        asm!("csrci mstatus, 8"); // Disable interrupts
        loop {
            asm!("wfi"); // Wait for interrupt
        }
    }
}

//#[cfg(not(test))]
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    hcf()
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}
