use core::clone::*;
use core::cmp::*;
use core::fmt::Debug;
use core::marker::Copy;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SMPInfo {
    pub hart_id: u64,
    pub cpus: &'static mut [&'static mut SMPInfoEntry],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SMPInfoEntry {
    /// ID of the processor
    ///
    /// ACPI Processor UID as specified by the MADT (always 0 on non-ACPI systems).
    pub processor_id: u64,
    /// HART ID of the processor
    ///
    /// Hart ID of the processor as specified by the MADT or Device Tree.
    pub hartid: u64,
    _reserved: u64, // reserved
    /// Address of the goto_address function
    ///
    /// An atomic write to this field causes the parked CPU to jump to the written address,
    /// on a 64KiB (or Stack Size Request size) stack. A pointer to the struct limine_smp_info structure of the CPU
    /// is passed in x10(a0).
    /// Other than that, the CPU state will be the same as described for the bootstrap processor.
    /// This field is unused for the structure describing the bootstrap processor.
    pub goto_address: extern "C" fn(&SMPInfoEntry) -> !,
    /// Free to use by the kernel
    pub extra: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelAddress {
    pub physical_address: u64,
    pub virtual_address: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMap {
    pub entries: &'static [&'static mut MemoryMapEntry],
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    /// The kernel and modules loaded are not marked as usable memory. They are
    /// marked as Kernel/Modules. The entries are guaranteed to be sorted by base
    /// address, lowest to highest. Usable and bootloader reclaimable entries are
    /// guaranteed to be 4096 byte aligned for both base and length. Usable and
    /// bootloader reclaimable entries are guaranteed not to overlap with any
    /// other entry. To the contrary, all non-usable entries (including kernel/modules)
    /// are not guaranteed any alignment, nor is it guaranteed that they do not
    /// overlap other entries.
    KernelAndModules = 6,
    Framebuffer = 7,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub memory_type: MemoryType,
}

pub struct EfiSystemTable {}
