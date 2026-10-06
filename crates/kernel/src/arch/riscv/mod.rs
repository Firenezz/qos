//! RISC-V 64-bit arch implementation + ELF/`EntryPointRequest` entry.

use core::num::NonZeroU64;

use log::{Level, log};

use crate::arch::device::uart::SbiLogger;
use crate::arch::{Arch, Memory, Processor};
use crate::console::Console;

mod entry;

pub mod device;
pub mod interrupt;
pub mod memory;

/// RISC-V 64-bit (RV64) target.
pub struct Riscv64;

/// BSP / application hart.
pub struct Hart;

impl Processor for Hart {
    #[inline(always)]
    fn wait_for_interrupt() {
        riscv_rs::operations::wfi();
    }

    fn halt() -> ! {
        // Prefer S-mode interrupt disable when running as Limine supervisor.
        // `csrci mstatus, 8` is M-mode; Limine hands off in S-mode — use sie clear.
        unsafe {
            core::arch::asm!("csrci sstatus, 2", options(nomem, nostack));
        }
        loop {
            Self::wait_for_interrupt();
        }
    }
}

impl Arch for Riscv64 {
    type Cpu = Hart;

    unsafe fn early_init() {
        use core::arch::asm;
        // TODO: trap vector, FP off, satp = bare, etc.
        unsafe {
            asm!(
                ".option push",
                ".option norelax",
                "la gp, __global_pointer$",
                ".option pop",
                options(nomem, nostack),
            );
        }
    }

    fn init(_info: &crate::boot::BootInfo) {
        // SAFETY: BSP bring-up; stack valid; IRQs still masked after this.

        Self::init_memory(_info);

        unsafe {
            interrupt::init();
        }
    }

    fn init_per_processor() {
        todo!()
    }

    fn init_logging(_uart_address: Option<NonZeroU64>) {
        crate::console::init(Console::Sbi(SbiLogger::new()));
        log!(Level::Info, "Initialized logging");
    }

    #[inline(always)]
    fn irq_enable() {
        interrupt::enable();
    }

    #[inline(always)]
    fn irq_disable() {
        interrupt::disable();
    }

    #[inline(always)]
    fn irq_is_enabled() -> bool {
        interrupt::is_enabled()
    }

    /// Register devices with the platform.
    ///
    /// This should probe and register boot-time devices like timers
    fn register_devices() {
        device::register_devices();
    }
}
