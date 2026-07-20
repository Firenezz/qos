pub mod memory;
pub mod processor;
pub mod alloc;
pub mod cpu;

use crate::arch::{Arch, Current};
use crate::boot::{BootInfo, Capability};

use uefi::system;

/// Bootloader-agnostic kernel entry after a protocol has produced [`BootInfo`].
pub fn kernel_main(info: BootInfo) -> ! {
    // SAFETY: BSP only; interrupts undefined; paging still bootloader's.

    match info.efi_system_table {
        Capability::Present(addr) => {
            unsafe {
                system::set_system_table(addr as *const system::SystemTable);
            }

            if let Some(system_table) = system::get_system_table() {
                // SAFETY: firmware configuration table list is readable.
                let _configuration_tables = unsafe { system_table.configuration_tables() };
            }
        }
        Capability::Absent => {}
    }
    log::info!("Initializing architecture specific state");
    Current::init(&info);

    log::info!("qos-kernel online");
    log::info!("Kernel version: {}", env!("CARGO_PKG_VERSION"));



    crate::halt()
}

pub fn kernel_secondary_main(info: Option<BootInfo>) -> ! {
    Current::init(&info.expect("Boot info is required for secondary processors"));

    crate::halt()
}

/// Pre-bootloader initialization.
/// This function is called before the bootloader mapping boot information.
/// It looks for an early logging device and initializes the logging system.
/// 
/// # Safety
/// It is safe to use any function that is available before the bootloader mapping boot information.
pub fn pre_bootloader_init() {
    Current::init_logging();
}