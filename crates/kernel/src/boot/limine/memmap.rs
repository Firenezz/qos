//! Limine memmap → [`crate::boot::memory`] traits.

use limine::memmap;

use crate::boot::memory::{BootMemoryKind, BootMemoryMap, BootMemoryRegion};
use crate::kernel::memory::MemoryKind;

use super::LimineBootInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimineMemoryKind(pub u64);

impl BootMemoryKind for LimineMemoryKind {
    fn to_memory_kind(self) -> MemoryKind {
        match self.0 {
            memmap::MEMMAP_USABLE => MemoryKind::Usable,
            memmap::MEMMAP_RESERVED => MemoryKind::Reserved,
            memmap::MEMMAP_ACPI_RECLAIMABLE => MemoryKind::AcpiReclaimable,
            memmap::MEMMAP_ACPI_NVS => MemoryKind::AcpiNvs,
            memmap::MEMMAP_BAD_MEMORY => MemoryKind::BadMemory,
            memmap::MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryKind::BootloaderReclaimable,
            memmap::MEMMAP_EXECUTABLE_AND_MODULES => MemoryKind::KernelAndModules,
            memmap::MEMMAP_FRAMEBUFFER => MemoryKind::Framebuffer,
            memmap::MEMMAP_MAPPED_RESERVED => MemoryKind::ReservedMapped,
            other => MemoryKind::Unknown(other),
        }
    }
}

impl LimineMemoryKind {
    pub const fn debug_name(self) -> &'static str {
        match self.0 {
            memmap::MEMMAP_USABLE => "usable",
            memmap::MEMMAP_RESERVED => "reserved",
            memmap::MEMMAP_ACPI_RECLAIMABLE => "acpi-reclaimable",
            memmap::MEMMAP_ACPI_NVS => "acpi-nvs",
            memmap::MEMMAP_BAD_MEMORY => "bad-memory",
            memmap::MEMMAP_BOOTLOADER_RECLAIMABLE => "bootloader-reclaimable",
            memmap::MEMMAP_EXECUTABLE_AND_MODULES => "executable-and-modules",
            memmap::MEMMAP_FRAMEBUFFER => "framebuffer",
            memmap::MEMMAP_MAPPED_RESERVED => "reserved-mapped",
            _ => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LimineMemoryRegion {
    base: u64,
    length: u64,
    kind: LimineMemoryKind,
}

impl BootMemoryRegion for LimineMemoryRegion {
    type Kind = LimineMemoryKind;

    fn base(&self) -> u64 {
        self.base
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn kind(&self) -> Self::Kind {
        self.kind
    }
}

impl BootMemoryMap for LimineBootInfo {
    type Region = LimineMemoryRegion;

    fn region_count(&self) -> usize {
        self.memmap_response.map(|m| m.entries().len()).unwrap_or(0)
    }

    fn region(&self, index: usize) -> Option<Self::Region> {
        let entry = self.memmap_response?.entries().get(index)?;
        Some(LimineMemoryRegion {
            base: entry.base,
            length: entry.length,
            kind: LimineMemoryKind(entry.type_),
        })
    }
}
