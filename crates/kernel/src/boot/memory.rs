//! Bootloader-agnostic memory map traits.

use crate::definitions::PhysicalAddress;
use crate::kernel::memory::{MemoryKind, MemoryRegion};

/// When a region may be promoted into the kernel's allocatable pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReclaimStage {
    /// Usable RAM at handoff.
    Immediate,
    /// After bootloader/firmware structures are no longer needed.
    BootloaderDone,
    /// After ACPI tables have been parsed.
    AcpiParsed,
}

/// Classifies a bootloader memmap entry and when it may be reclaimed.
pub trait BootMemoryKind: Copy {
    fn to_memory_kind(self) -> MemoryKind;

    /// `Some(stage)` when the region can eventually be allocated from;
    /// `None` when it must never enter the allocatable pool.
    fn reclaim_stage(self) -> Option<ReclaimStage> {
        self.to_memory_kind().reclaim_stage()
    }
}

/// One physical memory region from a bootloader map.
pub trait BootMemoryRegion {
    type Kind: BootMemoryKind;

    fn base(&self) -> u64;
    fn length(&self) -> u64;
    fn kind(&self) -> Self::Kind;

    fn to_memory_region(self) -> MemoryRegion
    where
        Self: Sized,
    {
        MemoryRegion::new(
            PhysicalAddress::new(self.base() as usize),
            self.length() as usize,
            self.kind().to_memory_kind(),
        )
    }
}

/// Full memory map provided by a bootloader.
pub trait BootMemoryMap {
    type Region: BootMemoryRegion;

    fn region_count(&self) -> usize;

    fn region(&self, index: usize) -> Option<Self::Region>;
}

impl<M: BootMemoryMap> BootMemoryMap for &M {
    type Region = M::Region;

    fn region_count(&self) -> usize {
        (*self).region_count()
    }

    fn region(&self, index: usize) -> Option<Self::Region> {
        (*self).region(index)
    }
}
