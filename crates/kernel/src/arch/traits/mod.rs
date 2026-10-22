use core::num::NonZeroU64;

use crate::definitions::PhysicalAddress;
use crate::definitions::VirtualAddress;

/// Per-CPU operations for the target ISA.
pub trait Processor {
    /// Wait for an interrupt (`wfi` / `hlt` / …).
    fn wait_for_interrupt();

    /// Park this CPU forever (interrupts off if applicable).
    fn halt() -> !;
}

/// Target architecture façade used by the rest of the kernel.
pub trait Arch: Sized {
    type Cpu: Processor;

    /// Very early arch setup after GP/stack are valid, before protocol handoff returns
    /// into [`crate::kernel::kernel_main`]. Safe to leave empty.
    ///
    /// # Safety
    /// Called once on the BSP with interrupt state undefined.
    unsafe fn early_init() {}

    /// Initialize the architecture specific state for the boot information.
    fn init(info: &crate::boot::BootInfo);

    /// Initialize the architecture specific state for each processor.
    ///
    /// This function is called for each processor after the APs entered the kernel main function.
    fn init_per_processor();

    /// Initialize the logging system.
    ///
    /// # Safety
    ///
    /// Called once by the BSP only
    fn init_logging(uart_address: Option<NonZeroU64>);

    /// Enable IRQs on this CPU.
    fn irq_enable();

    /// Disable IRQs on this CPU.
    fn irq_disable();

    /// Whether IRQs are enabled on this CPU.
    fn irq_is_enabled() -> bool;

    #[inline(always)]
    fn halt() -> ! {
        Self::Cpu::halt()
    }

    fn with_disabled_irqs<F: FnOnce() -> R, R>(f: F) -> R {
        let were_enabled = Self::irq_is_enabled();
        Self::irq_disable();
        let result = f();
        if were_enabled {
            Self::irq_enable();
        }
        result
    }

    /// Register devices with the platform.
    ///
    /// This should probe and register boot-time devices like timers
    fn register_devices();
}

pub trait Memory {
    /// Initialize the memory system.
    fn init_memory(info: &crate::boot::BootInfo);

    fn init_allocator();

    /// Get the highest table level.
    fn get_highest_table_level() -> usize;

    /// Get the physical address of the highest table level.
    fn get_highest_table_physical_address() -> PhysicalAddress;

    /// Get the virtual address of the highest table level.
    fn get_highest_table_virtual_address() -> VirtualAddress;

    fn set_hhdm_offset(offset: u64);

    fn get_hhdm_offset() -> u64;

    fn get_hhdm_mapped_physical_address(physical_address: PhysicalAddress) -> VirtualAddress;
}
