use core::sync::atomic::{AtomicPtr, Ordering};


#[repr(C)]
#[derive(Debug, Clone)]
pub struct SMPInfo {
    pub hart_id: u64,
    pub cpus: &'static [&'static mut Cpu],
}

/// A function pointer that the core will jump to when it is written to.
#[repr(transparent)]
#[derive(Debug)]
pub struct GotoAddress {
    inner: AtomicPtr<()>,
}
impl GotoAddress {
    /// Set the goto address pointer. This will cause the core to jump to the
    /// given function. This function also synchronizes all writes, so anything
    /// written before this function returns is guaranteed to be seen before the
    /// core jumps to the given function.
    pub fn write(&self, func: unsafe extern "C" fn(&Cpu) -> !) {
        self.inner.store(func as *mut (), Ordering::Release);
    }
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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub memory_type: MemoryType,
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

    Other
}

/// A CPU entry in the SMP request.
#[cfg(target_arch = "riscv64")]
#[repr(C)]
#[derive(Debug)]
pub struct Cpu {
    /// The ACPI processor ID, according to the ACPI MADT.
    pub id: u64,
    /// The hart ID, according to the ACPI MADT or the device tree.
    pub hartid: u64,
    _reserved: core::mem::MaybeUninit<u64>,
    /// The address to jump to. Writing to this field will cause the core to
    /// jump to the given function. The function will receive a pointer to this
    /// structure, and it will have its own 64KiB (or requested-size) stack.
    pub goto_address: GotoAddress,
    /// Free for use by the kernel.
    pub extra: u64,
}

pub struct EfiSystemTable {}