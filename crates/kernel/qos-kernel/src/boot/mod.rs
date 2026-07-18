//! Boot protocols → [`BootInfo`] → [`kernel_main`].
//!
//! Sequence (all bootloaders):
//! 1. `boot_main` (arch) — set GP
//! 2. [`early_start`] — pick protocol adapter
//! 3. `protocol::handoff` — build [`BootInfo`]
//! 4. [`kernel_main`] — shared kernel

pub mod capabilities;
pub mod info;

#[cfg(feature = "boot-limine")]
pub mod limine;

pub use capabilities::Capability;
pub use info::{
    Architecture, BootInfo, CpuInfo, CpuMap, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion,
};
use uefi::{guid, proto::acpi::AcpiTable, table::cfg::ConfigTableEntry};

/// Bootloader-agnostic kernel entry after a protocol has produced [`BootInfo`].
pub fn kernel_main(info: BootInfo) -> ! {
    // SAFETY: BSP only; interrupts undefined; paging still bootloader's.
    match info.efi_system_table {
        Capability::Present(addr) => {
            unsafe {
                uefi::table::set_system_table(addr as *const uefi_raw::table::system::SystemTable);
            }
        }
        _ => unreachable!("UEFI system table not present"),
    }
    crate::halt()
}