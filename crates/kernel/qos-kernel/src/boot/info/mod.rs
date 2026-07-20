//! Bootloader-agnostic boot information.
//!
//! Limine (or another boot protocol) converts into this via [`From`] / [`TryFrom`].
//! Optional firmware services use [`Capability`].

use crate::boot::capabilities::Capability;

/// Snapshot of what the bootloader handed the kernel.
///
/// Pointers/slices that come from bootloader memory are only valid until that
/// memory is reclaimed.
#[derive(Debug, Clone)]
pub struct BootInfo {
    pub architecture: Architecture,

    /// Hardware id of the bootstrap CPU (hart id, APIC id, …).
    pub bsp_hw_id: u64,

    /// Physical base of the kernel image, if known.
    pub kernel_phys: Capability<u64>,
    /// Virtual base of the kernel image, if known.
    pub kernel_virt: Capability<u64>,

    /// Higher-half direct map offset (phys + offset → virt), if provided.
    pub hhdm_offset: Capability<u64>,

    pub memory: Capability<MemoryMap>,
    pub cpus: Capability<CpuMap>,

    pub framebuffer: Capability<FramebufferInfo>,
    /// Physical address of the ACPI RSDP, if any.
    pub rsdp: Capability<u64>,
    /// Virtual pointer to an FDT blob, if any.
    pub dtb: Capability<*const u8>,
    /// Virtual/physical pointer to the EFI system table, if any.
    pub efi_system_table: Capability<*const ()>,
}

impl BootInfo {
    pub const fn empty() -> Self {
        Self {
            architecture: Architecture::Unknown,
            bsp_hw_id: 0,
            kernel_phys: Capability::Absent,
            kernel_virt: Capability::Absent,
            hhdm_offset: Capability::Absent,
            memory: Capability::Absent,
            cpus: Capability::Absent,
            framebuffer: Capability::Absent,
            rsdp: Capability::Absent,
            dtb: Capability::Absent,
            efi_system_table: Capability::Absent,
        }
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    RiscV64,
    Other(&'static str),
    Unknown,
}

impl Architecture {
    pub const fn native() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            Self::Aarch64
        }
        #[cfg(target_arch = "riscv64")]
        {
            Self::RiscV64
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        {
            Self::Unknown
        }
    }
}

/// Memory map as reported by the bootloader (often bootloader-reclaimable).
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub regions: &'static [MemoryRegion],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryKind,
}

/*
* `LIMINE_MEMMAP_USABLE` entries represent regions of the address space that are usable RAM,
and do not contain other data, the executable, bootloader information, or anything valuable,
and are therefore free for use.

* `LIMINE_MEMMAP_RESERVED` entries represent regions of the address space that are
reserved for unspecified purposes by the firmware, hardware, or otherwise, and should not
be touched by the executable.

* `LIMINE_MEMMAP_ACPI_RECLAIMABLE` entries represent regions of the address space containing
ACPI related data, such as ACPI tables and AML code. The executable should make absolutely
sure that no data contained in these regions is still needed before deciding to reclaim
these memory regions for itself. Refer to the ACPI specification for further information.

* `LIMINE_MEMMAP_ACPI_NVS` entries represent regions of the address space used for ACPI
non-volatile data storage. Refer to the ACPI specification for further information.

* `LIMINE_MEMMAP_BAD_MEMORY` entries represent regions of the address space that contain
bad RAM, which may be unreliable, and therefore these regions should be treated the same
as reserved regions.

* `LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE` entries represent regions of the address space
containing RAM used to store bootloader or firmware information that should be available
to the executable (or, in some cases, hardware, such as for MP trampolines). The executable
should make absolutely sure that no data contained in these regions is still needed before
deciding to reclaim these memory regions for itself.

* `LIMINE_MEMMAP_EXECUTABLE_AND_MODULES` entries are meant to have an illustrative purpose
only, and are not authoritative sources to be used as a means to find the addresses of the
executable or modules. One must use the specific Limine features ([Executable Address](#executable-address-feature) and
[Module](#module-feature) features) to do that.

* `LIMINE_MEMMAP_FRAMEBUFFER` entries represent regions of the address space containing
memory-mapped framebuffers. These entries exist for illustrative purposes only, and are
not to be used to acquire the address of any framebuffer. One must use the [Framebuffer
feature](#framebuffer-feature) for that.

* `LIMINE_MEMMAP_RESERVED_MAPPED` ([base revision 4](#base-revision-4) or greater) entries represent regions
of the address space containing the ACPI tables as described by the [Memory Layout at Entry](#memory-layout-at-entry)
section, if the firmware did not already map them within either an ACPI reclaimable
or an ACPI NVS region. For [base revision 5](#base-revision-5) or greater, these entries additionally
contain SMBIOS tables, EFI Runtime Services code and data, and the EFI system table along
with the data it references (see [Base Revision 5](#base-revision-5) for details).
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// Usable memory.
    /// This memory is usable by the kernel and modules.
    Usable,
    /// Reserved memory.
    /// This memory is reserved by the bootloader and is not usable by the kernel and modules.
    /// 
    /// # Safety
    /// 
    /// This memory is reserved by the bootloader and is not usable by the kernel and modules.
    /// The bootloader should make absolutely sure that no data contained in these regions is still
    /// needed before deciding to reserve these memory regions for itself.
    Reserved,
    /// ACPI reclaimable memory.
    /// 
    /// # Description
    /// ACPI reclaimable memory is used for ACPI related data, such as ACPI tables and AML code. Refer to the ACPI specification for further information.
    /// ([ACPI specification](https://uefi.org/sites/default/files/resources/ACPI_6_3.pdf))
    /// 
    /// # Safety
    /// This memory is reclaimable by the kernel and modules. The kernel should make absolutely
    /// sure that no data contained in these regions is still needed before deciding to reclaim
    /// these memory regions for itself. Refer to the ACPI specification for further information.
    AcpiReclaimable,
    /// ACPI NVS memory.
    /// 
    /// # Description
    /// ACPI NVS memory is used for ACPI non-volatile data storage. Refer to the ACPI specification for further information.
    /// ([ACPI specification](https://uefi.org/sites/default/files/resources/ACPI_6_3.pdf))
    /// 
    /// # Safety
    /// This memory is used for ACPI NVS and is not usable by the kernel and modules.
    /// The kernel should make absolutely sure that no data contained in these regions is still
    /// needed before deciding to use these memory regions for itself. Refer to the ACPI specification for further information.
    AcpiNvs,
    /// Bad memory.
    /// 
    /// # Description
    /// Bad memory is memory that have previously marked as bad or unreliable by the firmware
    /// It should be treated the same as reserved memory.
    /// 
    /// # Safety
    /// The kernel should make sure that no data contained in these regions is still needed.
    /// Usage of this memory needs to be absolutely minimal and should be avoided if possible.
    /// Usage of this memory could lead to undefined behavior or lost of data.
    BadMemory,
    /// Bootloader reclaimable memory.
    /// 
    /// # Description
    /// Bootloader reclaimable memory is memory that is reclaimable by the kernel and modules.
    /// It is free for use when the kernel decides that its data is no longer needed.
    /// 
    /// # Remarks
    /// This memory could be the requests from limine protocol or other firmware services.
    /// 
    /// # Safety
    /// The kernel should make sure that no data contained in these regions is still needed.
    BootloaderReclaimable,
    /// Kernel and modules memory.
    /// 
    /// # Description
    /// Kernel and modules memory is memory that is used for the kernel and modules.
    /// This is informative only and shouldnt be used to acquire the address of any kernel or module.
    KernelAndModules,
    /// Framebuffer memory.
    /// 
    /// # Description
    /// Framebuffer memory is memory that is used for the framebuffer.
    /// This is informative only and shouldnt be used to acquire the address of any framebuffer.
    Framebuffer,
    /// Reserved mapped memory.
    /// This memory is reserved by the bootloader and is not available for use by the kernel and modules.
    ReservedMapped,
    /// Unknown memory.
    /// This memory is unknown and is not available for use by the kernel and modules.
    Unknown(u64),
}

impl MemoryKind {
    pub const fn from_u64(value: u64) -> Self {
        match value {
            0 => Self::Usable,
            1 => Self::Reserved,
            2 => Self::AcpiReclaimable,
            3 => Self::AcpiNvs,
        }
    }

    pub const fn can_freely_be_reclaimed(self) -> bool {
        matches!(self, 
            Self::Usable |
            Self::AcpiReclaimable |
            Self::BootloaderReclaimable
        )
    }
}

/// CPU list as reported by the bootloader.
#[derive(Debug, Clone, Copy)]
pub struct CpuMap {
    pub cpus: &'static [CpuInfo],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuInfo {
    /// Dense logical index assigned by the kernel later (0 = BSP once assigned).
    pub cpu_id: u32,
    /// Architecture hardware id (hart id, LAPIC id, MPIDR, …).
    pub hw_id: u64,
    /// ACPI Processor UID when present.
    pub acpi_uid: Capability<u32>,
    pub is_bsp: bool,
}

impl CpuInfo {
    pub const fn new(cpu_id: u32, hw_id: u64, acpi_uid: Capability<u32>, is_bsp: bool) -> Self {
        Self { cpu_id, hw_id, acpi_uid, is_bsp }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
}
