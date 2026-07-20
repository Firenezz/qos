//! Limine points [`EntryPointRequest`] at the shared [`crate::arch::riscv::boot_main`].

use limine::{mp::MpInfo, request::EntryPointRequest};

use crate::{arch::*, halt};

/// Same entry as ELF `e_entry` — one GP setup, then [`crate::boot::early_start`].
#[used]
#[cfg(not(any(target_arch = "riscv64")))]
#[unsafe(link_section = ".requests")]
error!("Unsupported architecture");

#[cfg(target_arch = "riscv64")]
#[unsafe(link_section = ".requests")]
pub static ENTRY_POINT_REQUEST: EntryPointRequest = EntryPointRequest::new(limine_entry);

extern "C" fn limine_entry() -> ! {
    unsafe {
        Current::early_init();
    }
    // call pre-bootloader initialization for early logging and other setup
    crate::kernel::pre_bootloader_init();
    // Call the handoff function that will map the boot information and enter the kernel main function
    super::handoff();
}

pub extern "C" fn limine_ap_trampoline(info: &MpInfo) -> ! {
    unsafe {
        Current::early_init();
    }

    halt();
}