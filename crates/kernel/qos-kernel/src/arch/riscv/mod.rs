//! RISC-V 64-bit arch implementation + ELF/`EntryPointRequest` entry.

use crate::arch::{Arch, Cpu};

mod entry;

pub mod device;
pub mod interrupt;

/// RISC-V 64-bit (RV64) target.
pub struct Riscv64;

/// BSP / application hart.
pub struct Hart;

impl Cpu for Hart {
    type Id = u64;

    #[inline(always)]
    fn id() -> Self::Id {
        // TODO: Limine hands off in S-mode (`mhartid` traps). Use SMP response /
        // SBI / `tp` once those are wired; BSP id also arrives via `BootInfo`.
        0
    }

    #[inline(always)]
    fn wait_for_interrupt() {
        // SAFETY: `wfi` is always valid to execute.
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
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
        unsafe {
            interrupt::init();
        }
    }

    fn init_logging() {
        unimplemented!()
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
}
