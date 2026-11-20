//! Bootloader-agnostic memory map traits and owned handoff snapshot.

use crate::definitions::PhysicalAddress;
use crate::kernel::memory::{MAX_SNAPSHOT_REGIONS, MemoryKind, MemoryRegion};

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

/// Owned copy of the bootloader memmap for handoff into [`crate::boot::BootInfo`].
///
/// Protocol adapters fill this; [`crate::kernel::kernel_main`] installs it into
/// the global startup memory map.
#[derive(Clone, Debug)]
pub struct BootMemorySnapshot {
    regions: [MemoryRegion; MAX_SNAPSHOT_REGIONS],
    len: usize,
}

impl BootMemorySnapshot {
    pub const fn empty() -> Self {
        Self {
            regions: [MemoryRegion::EMPTY; MAX_SNAPSHOT_REGIONS],
            len: 0,
        }
    }

    pub fn from_boot_map<M: BootMemoryMap>(boot: &M) -> Self {
        let mut regions = [MemoryRegion::EMPTY; MAX_SNAPSHOT_REGIONS];
        let count = boot.region_count().min(MAX_SNAPSHOT_REGIONS);
        for i in 0..count {
            let Some(region) = boot.region(i) else {
                break;
            };
            regions[i] = region.to_memory_region();
        }
        Self {
            regions,
            len: count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }
}

impl BootMemoryMap for BootMemorySnapshot {
    type Region = MemoryRegion;

    fn region_count(&self) -> usize {
        self.len
    }

    fn region(&self, index: usize) -> Option<Self::Region> {
        self.regions().get(index).copied()
    }
}

impl BootMemoryKind for MemoryKind {
    fn to_memory_kind(self) -> MemoryKind {
        self
    }
}

impl BootMemoryRegion for MemoryRegion {
    type Kind = MemoryKind;

    fn base(&self) -> u64 {
        MemoryRegion::base(self).as_usize() as u64
    }

    fn length(&self) -> u64 {
        MemoryRegion::length(self) as u64
    }

    fn kind(&self) -> Self::Kind {
        self.kind
    }
}
