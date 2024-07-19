#![no_std]
#![no_main]

/// Remember that when response is None, it means that the request has not been responded to yet or failed.
pub mod common;

static STACK: [u8; 4096] = MaybeUninit::zeroed();

use core::mem::MaybeUninit;

use common::create_limine_common_boot_info;
use common_boot::BootInfo;
use limine::smp::Cpu;

use qos_kernel::hcf;

#[export_name = "boot_main"]
#[no_mangle]
#[allow(unreachable_code)]
/// This is the entry point for the kernel.
///
/// State of the CPU:
///
///
unsafe extern "C" fn _start() -> ! {
    let test = create_limine_common_boot_info();

    let boot_info = common_boot::BootInfo::new_from_limine(test);

    qos_kernel::kernel_main(boot_info);

    hcf()
}

extern "C" fn secondary_main(info: &Cpu) -> ! {
    hcf()
}
