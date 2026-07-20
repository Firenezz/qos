



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

    pub const fn can_freely_be_reclaimed(self) -> bool {
        matches!(self, 
            Self::Usable |
            Self::AcpiReclaimable |
            Self::BootloaderReclaimable
        )
    }

    pub const fn is_reserved(self) -> bool {
        matches!(self, Self::Reserved | Self::BadMemory | Self::ReservedMapped)
    }

    pub const fn is_acpi_reclaimable(self) -> bool {
        matches!(self, Self::AcpiReclaimable | Self::AcpiNvs)
    }

    pub const fn is_bad_memory(self) -> bool {
        matches!(self, Self::BadMemory)
    }

    pub const fn is_framebuffer(self) -> bool {
        matches!(self, Self::Framebuffer)
    }

    pub const fn is_kernel_and_modules(self) -> bool {
        matches!(self, Self::KernelAndModules)
    }

    pub const fn is_unknown(self) -> Option<u64> {
        if let Self::Unknown(value) = self {
            Some(value)
        } else {
            None
        }
    }
    
}