#![no_std]
#![no_main]

/// Remember that when response is None, it means that the request has not been responded to yet or failed.
mod limine_request;

static STACK: [u8; 4096] = [0; 4096];

use limine::SmpInfo;
use limine_request::*;

#[export_name = "boot_main"]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {

    if let (
        Some(smp_response),
        Some(kernel_address),
        Some(memory_map)) =
    (
        SMP_REQUEST.get_response().get_mut(),
        KERNEL_ADDRESS_REQUEST.get_response().get(),
        MEMMAP_REQUEST.get_response().get_mut()
    ) {

        let _memory_count = memory_map.entry_count;
        let _entries = memory_map.memmap();

        let _smp_count = smp_response.cpu_count;
        let _cpus = &smp_response.cpus;
        let _smp_cpus = smp_response.cpus();

        for cpu in _smp_cpus {
            cpu.goto_address = secondary_main;
        }

        panic!("I panic with passionate riscv");

    }

    // TODO: Call kernel_main().

    hcf()
}

use core::{panic, arch::asm};
#[cfg(not(test))]
use core::panic::PanicInfo;

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &PanicInfo) -> ! {
    hcf()
}

#[allow(dead_code)]
#[inline(never)]
fn hcf() -> ! {
    unsafe {
        asm!("csrci mstatus, 8"); // Disable interrupts
        loop {
            asm!("wfi"); // Wait for interrupt
        }
    }
}

extern "C" fn secondary_main(info: *const SmpInfo) -> ! {
    let _info = unsafe { &*info };
    hcf()
}