//! Limine points [`EntryPointRequest`] at the shared [`crate::arch::riscv::boot_main`].

use limine::{mp::MpInfo, request::EntryPointRequest};

use crate::{arch::*, halt, kernel::{kernel_main, kernel_secondary_main}};

/// Same entry as ELF `e_entry` — one GP setup, then [`crate::boot::early_start`].
#[used]
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
error!("Unsupported architecture");

#[cfg(any(target_arch = "riscv64", target_arch = "x86_64"))]
#[unsafe(link_section = ".requests")]
pub static ENTRY_POINT_REQUEST: EntryPointRequest = EntryPointRequest::new(limine_entry);

extern "C" fn limine_entry() -> ! {
    unsafe {
        Current::early_init();
    }
    // Call the handoff function that will map the boot information and enter the kernel main function
    super::handoff();
}

pub extern "C" fn limine_ap_trampoline(info: &MpInfo) -> ! {
    unsafe {
        Current::early_init();
    }
    #[cfg(target_arch = "riscv64")]
    log::info!("AP trampoline reached by processor {}", info.hartid);
    #[cfg(target_arch = "x86_64")]
    log::info!("AP trampoline reached by processor {}", info.lapic_id);
    log::info!("AP trampoline reached by acpi processor id {}", info.processor_id);

    //kernel_secondary_main(info);

    halt();
}