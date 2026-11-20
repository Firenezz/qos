//! Bootloader-agnostic kernel entry after protocol handoff.

use core::num::NonZeroU64;
use core::ptr::read_volatile;

use ::alloc::boxed::Box;
use riscv_rs::csr::satp;
use spin::Mutex;

use crate::arch::{Arch, Current, traits::Memory};
use crate::boot::BootInfo;
use crate::kernel::os::Os;
use crate::platform;

/// The global OS instance.
///
/// # Safety
/// Memory heap must be initialized before this is used.
static OS: spin::LazyLock<&'static Mutex<Os>> =
    spin::LazyLock::new(|| Box::leak(Box::new(Mutex::new(Os::new()))));

/// Bootloader-agnostic kernel entry after a protocol has produced [`BootInfo`].
pub fn kernel_main(info: BootInfo) -> ! {
    // SAFETY: BSP only; interrupts undefined; paging still bootloader's.
    log::info!("Initializing architecture specific state");
    if let Some(hhdm_offset) = info.hhdm_offset {
        <Current as Memory>::set_hhdm_offset(hhdm_offset);
    }

    if !info.memory.is_empty() {
        log::trace!("Installing boot memory map");
        crate::startup::memory::init_from_boot(&info.memory);
        crate::startup::memory::with_memory(|map| map.dump());
    }

    log::debug!("Boot info: {:#?}", info);
    Current::init(&info);

    // Platform bring-up is staged: firmware capture → arch service probe → publish.
    let mut platform_init = platform::Init::new();
    platform_init.capture_firmware(&info);
    // TODO: Current::probe_platform(&mut platform_init) — timer/IPI backends
    platform_init.finish();

    unsafe { riscv_rs::operations::ebreak() };

    log::info!("Flushed TLB");

    log::info!("qos-kernel online");
    log::info!("Kernel version: {}", env!("CARGO_PKG_VERSION"));

    let test = satp::read_raw();

    log::debug!("SATP: {:x}", test);

    let satp_test = satp::read();

    log::debug!("SATP PPN: {:x}", satp_test.ppn());
    log::debug!(
        "SATP page table physical address: {:x}",
        satp_test.ppn() << 12
    );
    log::debug!("SATP ASID: {:x}", satp_test.asid());
    log::debug!("SATP MODE: {:?}", satp_test.mode());

    crate::halt()
}

/// Secondary kernel main function.
///
/// Called by secondary processors to register with the kernel, initialize
/// per-processor arch state, then halt.
///
/// # Safety
/// [`Os`] (at least the processor map) and the memory map must already be initialized.
pub fn kernel_secondary_main(info: Option<crate::boot::CpuInfo>) -> ! {
    // TODO: register processor with the kernel and get its index

    OS.lock()
        .register_processor(info.expect("Processor info is required for secondary processors"));

    Current::init_per_processor();

    crate::halt()
}

/// Pre-bootloader initialization.
///
/// Called before bootloader mapping boot information is fully consumed.
/// Looks for an early logging device and initializes the logging system.
///
/// # Safety
/// Only functions available before boot info mapping may be used.
pub fn pre_bootloader_init(hhdm_offset: Option<NonZeroU64>) {
    Current::init_logging(hhdm_offset);
}
