use crate::arch::{Arch, Current};
use crate::boot::{BootInfo, Capability};

use uefi::system;

/// Bootloader-agnostic kernel entry after a protocol has produced [`BootInfo`].
pub fn kernel_main(info: BootInfo) -> ! {
    // SAFETY: BSP only; interrupts undefined; paging still bootloader's.
    Current::init(&info);

    match info.efi_system_table {
        Capability::Present(addr) => {
            // SAFETY: Limine hands off a mapped EFI_SYSTEM_TABLE address.
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

    Current::init(&info);

    unsafe { core::ptr::read_volatile(1 as *const u8); }

    crate::halt()
}