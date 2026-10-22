//! Classified physical memory regions.

use crate::boot::memory::ReclaimStage;
use crate::definitions::{PhysicalAddress, PhysicalAddressRange};

/// A physical MMIO / device memory range.
pub type DeviceMemoryRegion = PhysicalAddressRange;

/// A classified physical memory region from the boot memmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub range: PhysicalAddressRange,
    pub kind: MemoryKind,
}

impl MemoryRegion {
    pub const EMPTY: Self = Self {
        range: PhysicalAddressRange::EMPTY,
        kind: MemoryKind::Reserved,
    };

    pub fn new(base: PhysicalAddress, length: usize, kind: MemoryKind) -> Self {
        Self {
            range: PhysicalAddressRange::from_base_len(base, length),
            kind,
        }
    }

    pub fn base(&self) -> PhysicalAddress {
        self.range.base()
    }

    pub fn length(&self) -> usize {
        self.range.len()
    }

    pub fn end_exclusive(&self) -> PhysicalAddress {
        self.range.end()
    }

    pub fn end_inclusive(&self) -> PhysicalAddress {
        self.range.end_inclusive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Usable,
    Device,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootloaderReclaimable,
    KernelAndModules,
    Framebuffer,
    ReservedMapped,
    Unknown(u64),
}

impl MemoryKind {
    /// When this region may be promoted into the allocatable pool.
    pub const fn reclaim_stage(self) -> Option<ReclaimStage> {
        match self {
            Self::Usable => Some(ReclaimStage::Immediate),
            Self::BootloaderReclaimable => Some(ReclaimStage::BootloaderDone),
            Self::AcpiReclaimable => Some(ReclaimStage::AcpiParsed),
            _ => None,
        }
    }

    pub const fn is_ram(self) -> bool {
        self.reclaim_stage().is_some()
    }

    pub const fn is_device(self) -> bool {
        matches!(self, Self::Device)
    }

    /// MMIO / firmware IO windows (HHDM-mapped reserved, framebuffer, NVS, devices).
    pub const fn is_io_mapped(self) -> bool {
        matches!(
            self,
            Self::Device | Self::ReservedMapped | Self::Framebuffer | Self::AcpiNvs
        )
    }

    /// Fixed regions from the boot memmap — not PMM, not generic device MMIO.
    ///
    /// Framebuffer details come from [`crate::boot::BootInfo::framebuffer`], not the device pool.
    pub const fn is_special(self) -> bool {
        matches!(
            self,
            Self::Framebuffer | Self::KernelAndModules | Self::AcpiNvs | Self::BadMemory
        )
    }

    /// Firmware-reserved address space. Not RAM; kept in the snapshot only until
    /// ACPI/DT explicitly registers MMIO via [`super::MemoryMap::register_device`].
    pub const fn is_firmware_reserved(self) -> bool {
        matches!(self, Self::Reserved | Self::ReservedMapped)
    }

    pub const fn is_reserved(self) -> bool {
        matches!(
            self,
            Self::Reserved
                | Self::BadMemory
                | Self::ReservedMapped
                | Self::Framebuffer
                | Self::KernelAndModules
                | Self::AcpiNvs
        )
    }

    pub const fn is_acpi_nvs(self) -> bool {
        matches!(self, Self::AcpiNvs)
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
