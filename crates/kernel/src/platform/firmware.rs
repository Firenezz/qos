//! Boot firmware inventory (UEFI / optional SBI pairing).
//!
//! This is discovery data, not a runtime driver. Prefer MMIO/CSR services over
//! calling into firmware after ExitBootServices.

use uefi::{BootServices, ConfigurationTable, RuntimeServices, SystemTable};

/// Firmware tables captured from the boot path (if any).
pub enum Firmware {
    Uefi(UefiFirmware),
}

/// How the kernel is talking to the machine at a high level.
///
/// `Sbi` is meaningful on RISC-V; other arches leave this as [`Environment::Firmware`]
/// or extend with their own variants later.
pub enum Environment {
    /// UEFI (and similar) without an SBI-like runtime.
    Firmware(Firmware),
    /// SBI present; may still carry copied UEFI tables from boot.
    Sbi(SbiEnvironment),
}

pub struct SbiEnvironment {
    pub firmware: Option<Firmware>,
}

impl SbiEnvironment {
    pub fn new(firmware: Option<Firmware>) -> Self {
        Self { firmware }
    }

    pub fn firmware(&self) -> Option<&Firmware> {
        self.firmware.as_ref()
    }
}

pub struct UefiFirmware {
    pub system_table: Option<&'static SystemTable>,
    pub runtime_services: Option<&'static RuntimeServices>,
    /// Cleared after ExitBootServices / once we treat firmware as runtime-only.
    pub boot_services: Option<&'static BootServices>,
    pub configuration_table: Option<&'static ConfigurationTable>,
}

// SAFETY: Firmware table pointers are shared read-mostly handoff state. The
// kernel must synchronize any concurrent use (and must not call Boot Services
// after ExitBootServices). UEFI structs contain raw pointers, so they are not
// auto Send/Sync.
unsafe impl Send for UefiFirmware {}
unsafe impl Sync for UefiFirmware {}

impl UefiFirmware {
    pub fn new(
        system_table: &'static SystemTable,
        runtime_services: &'static RuntimeServices,
        boot_services: Option<&'static BootServices>,
        configuration_table: Option<&'static ConfigurationTable>,
    ) -> Self {
        Self {
            system_table: Some(system_table),
            runtime_services: Some(runtime_services),
            boot_services,
            configuration_table,
        }
    }

    pub fn drop_boot_services(&mut self) {
        self.boot_services = None;
    }
}
