//! Platform inventory + registered services.
//!
//! Bring-up is a **process** ([`Init`]), not a single assignment:
//!
//! 1. [`Init::new`] — empty builder
//! 2. [`Init::capture_firmware`] — UEFI / boot tables from [`BootInfo`]
//! 3. Arch probe — prefer MMIO/CSR → SBI-like → firmware; [`Init::register_timer`] /
//!    [`Init::register_ipi`]
//! 4. [`Init::finish`] — publish [`State`] to the global
//!
//! After finish, generic code uses [`timer`] / [`ipi`] only.

mod firmware;
mod ipi;
mod timer;

pub use firmware::{Environment, Firmware, SbiEnvironment, UefiFirmware};
pub use ipi::Ipi;
pub use timer::Timer;

use crate::boot::{BootInfo, Capability};
use spin::Mutex;
use uefi::system;

/// Long-lived platform root (firmware inventory + active service slots).
pub struct State {
    pub environment: Environment,
    timer: Option<&'static dyn Timer>,
    ipi: Option<&'static dyn Ipi>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// In-progress platform bring-up. Commit with [`Init::finish`].
pub struct Init {
    environment: Option<Environment>,
    timer: Option<&'static dyn Timer>,
    ipi: Option<&'static dyn Ipi>,
}

impl Init {
    pub const fn new() -> Self {
        Self {
            environment: None,
            timer: None,
            ipi: None,
        }
    }

    /// Step: capture firmware / environment hints from boot handoff.
    ///
    /// Does not register timer/IPI — that is an arch probe step afterward.
    pub fn capture_firmware(&mut self, info: &BootInfo) -> &mut Self {
        let uefi = match info.efi_system_table {
            Capability::Present(addr) => capture_uefi(addr),
            Capability::Absent => None,
        };

        // RISC-V kernels typically have SBI; other arches use Firmware-only.
        #[cfg(target_arch = "riscv64")]
        {
            self.environment = Some(Environment::Sbi(SbiEnvironment::new(
                uefi.map(Firmware::Uefi),
            )));
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            self.environment = uefi.map(|fw| Environment::Firmware(Firmware::Uefi(fw)));
        }

        if self.environment.is_some() {
            log::debug!("platform: firmware/environment captured");
        } else {
            log::debug!("platform: no firmware tables in BootInfo");
        }
        self
    }

    /// Step: override / set environment explicitly (e.g. tests or late fixups).
    pub fn set_environment(&mut self, environment: Environment) -> &mut Self {
        self.environment = Some(environment);
        self
    }

    /// Step: arch binds the preferred timer backend.
    pub fn register_timer(&mut self, timer: &'static dyn Timer) -> &mut Self {
        self.timer = Some(timer);
        log::debug!("platform: timer selected");
        self
    }

    /// Step: arch binds the preferred IPI backend.
    pub fn register_ipi(&mut self, ipi: &'static dyn Ipi) -> &mut Self {
        self.ipi = Some(ipi);
        log::debug!("platform: ipi selected");
        self
    }

    pub fn timer_slot(&self) -> Option<&'static dyn Timer> {
        self.timer
    }

    pub fn ipi_slot(&self) -> Option<&'static dyn Ipi> {
        self.ipi
    }

    /// Final step: publish to the global. Fails soft if environment was never set.
    pub fn finish(self) {
        let Some(environment) = self.environment else {
            log::error!("platform: finish without environment — call capture_firmware first");
            return;
        };

        let state = State {
            environment,
            timer: self.timer,
            ipi: self.ipi,
        };

        let mut guard = STATE.lock();
        if guard.is_some() {
            log::warn!("platform: finish called more than once");
            return;
        }
        *guard = Some(state);
        log::info!("platform: ready");
    }
}

fn capture_uefi(addr: *const ()) -> Option<UefiFirmware> {
    // SAFETY: BootInfo advertises a system table pointer from the bootloader.
    unsafe {
        system::set_system_table(addr as *const system::SystemTable);
    }

    let system_table = system::get_system_table()?;
    // SAFETY: pointer came from firmware/bootloader handoff.
    if unsafe { !system_table.is_valid() } {
        log::error!("platform: EFI system table failed validation");
        return None;
    }

    let runtime_services = non_null(system_table.RuntimeServices)?;
    let boot_services = non_null(system_table.BootServices);
    let configuration_table = non_null(system_table.ConfigurationTables);

    Some(UefiFirmware::new(
        system_table,
        runtime_services,
        boot_services,
        configuration_table,
    ))
}

fn non_null<T>(ptr: *mut T) -> Option<&'static T> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: non-null firmware table pointer from a validated system table.
        Some(unsafe { &*ptr })
    }
}

fn with_state<R>(f: impl FnOnce(&State) -> R) -> Option<R> {
    STATE.lock().as_ref().map(f)
}

pub fn timer() -> Option<&'static dyn Timer> {
    with_state(|s| s.timer).flatten()
}

pub fn ipi() -> Option<&'static dyn Ipi> {
    with_state(|s| s.ipi).flatten()
}

pub fn with_environment<R>(f: impl FnOnce(&Environment) -> R) -> Option<R> {
    with_state(|s| f(&s.environment))
}

pub fn is_ready() -> bool {
    STATE.lock().is_some()
}
