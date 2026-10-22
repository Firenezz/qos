//! Kernel memory map: boot snapshot plus managed allocatable regions.

use core::fmt::{self, Arguments};

use crate::boot::memory::ReclaimStage;
use crate::definitions::PhysicalAddress;

pub mod frame;
pub mod region;

pub use region::{DeviceMemoryRegion, MemoryKind, MemoryRegion};

pub const MAX_SNAPSHOT_REGIONS: usize = 128;
pub const MAX_ALLOCATABLE_REGIONS: usize = 64;

/// Immutable bootloader memmap snapshot plus the kernel's managed region pool.
#[derive(Clone, Debug)]
pub struct MemoryMap {
    snapshot: ([MemoryRegion; MAX_SNAPSHOT_REGIONS], usize),
    /// RAM available to the PMM and registered device/MMIO regions (`MemoryKind::Device`).
    allocatable: ([MemoryRegion; MAX_ALLOCATABLE_REGIONS], usize),
    reclaim_stage: ReclaimStage,
}

impl MemoryMap {
    pub const fn empty() -> Self {
        Self {
            snapshot: ([MemoryRegion::EMPTY; MAX_SNAPSHOT_REGIONS], 0),
            allocatable: ([MemoryRegion::EMPTY; MAX_ALLOCATABLE_REGIONS], 0),
            reclaim_stage: ReclaimStage::Immediate,
        }
    }

    pub fn from_snapshot(snapshot: [MemoryRegion; MAX_SNAPSHOT_REGIONS], len: usize) -> Self {
        let mut map = Self::empty();
        map.snapshot = (snapshot, len.min(MAX_SNAPSHOT_REGIONS));
        map.promote_through(ReclaimStage::Immediate);
        map
    }

    pub fn snapshot(&self) -> &[MemoryRegion] {
        &self.snapshot.0[..self.snapshot.1]
    }

    pub fn allocatable(&self) -> &[MemoryRegion] {
        &self.allocatable.0[..self.allocatable.1]
    }

    pub fn ram<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.allocatable
            .0
            .iter()
            .take(self.allocatable.1)
            .filter(|region| region.kind.is_ram())
    }

    pub fn devices<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.allocatable
            .0
            .iter()
            .take(self.allocatable.1)
            .filter(|region| region.kind.is_device())
    }

    /// Snapshot / registered regions that represent MMIO or firmware IO windows.
    pub fn io_mapped<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.snapshot()
            .iter()
            .filter(|region| region.kind.is_io_mapped())
            .chain(self.devices())
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.allocatable().iter()
    }

    pub fn snapshot_region_at(&self, address: PhysicalAddress) -> Option<MemoryRegion> {
        self.snapshot()
            .iter()
            .copied()
            .find(|region| region.range.contains(address))
    }

    pub fn is_allocatable(&self, address: PhysicalAddress) -> bool {
        self.ram().any(|region| region.range.contains(address))
    }

    pub fn device_contains(&self, address: PhysicalAddress) -> bool {
        self.devices().any(|region| region.range.contains(address))
    }

    pub fn device_region_at(&self, address: PhysicalAddress) -> Option<MemoryRegion> {
        self.devices()
            .copied()
            .find(|region| region.range.contains(address))
    }

    pub fn reclaim_stage(&self) -> ReclaimStage {
        self.reclaim_stage
    }

    pub fn usable_regions<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.allocatable
            .0
            .iter()
            .take(self.allocatable.1)
            .filter(|region| region.kind == MemoryKind::Usable)
    }

    pub fn non_usable_regions<'a>(&'a self) -> impl Iterator<Item = &'a MemoryRegion> {
        self.allocatable
            .0
            .iter()
            .take(self.allocatable.1)
            .filter(|region| region.kind.is_ram() && region.kind != MemoryKind::Usable)
    }

    pub fn kernel(&self) -> Option<&MemoryRegion> {
        self.snapshot()
            .iter()
            .find(|region| region.kind.is_kernel_and_modules())
    }

    /// Register a physical MMIO range (e.g. from ACPI or a driver).
    pub fn register_device(&mut self, base: PhysicalAddress, length: usize) -> bool {
        let region = MemoryRegion::new(base, length, MemoryKind::Device);
        if region.range.is_empty() {
            return false;
        }

        if self.ram().any(|ram| region.range.overlaps(ram.range)) {
            return false;
        }

        if self
            .snapshot()
            .iter()
            .any(|entry| region.range.overlaps(entry.range) && entry.kind.is_special())
        {
            return false;
        }

        self.insert_device(region)
    }

    /// Rebuild RAM regions from the snapshot; registered device regions are preserved.
    pub fn promote_through(&mut self, stage: ReclaimStage) {
        let mut devices = [MemoryRegion::EMPTY; MAX_ALLOCATABLE_REGIONS];
        let mut device_len = 0;
        for region in self.allocatable() {
            if region.kind.is_device() && device_len < MAX_ALLOCATABLE_REGIONS {
                devices[device_len] = *region;
                device_len += 1;
            }
        }

        self.allocatable.1 = 0;
        for i in 0..self.snapshot.1 {
            let region = self.snapshot.0[i];
            if region
                .kind
                .reclaim_stage()
                .is_some_and(|required| required <= stage)
            {
                self.insert_ram(region);
            }
        }

        for i in 0..device_len {
            let _ = self.insert_device(devices[i]);
        }
        self.reclaim_stage = stage;
    }

    fn insert_ram(&mut self, region: MemoryRegion) {
        if region.range.is_empty() || !region.kind.is_ram() {
            return;
        }

        if self.allocatable.1 > 0 {
            let last = &mut self.allocatable.0[self.allocatable.1 - 1];
            if last.kind.is_ram() && region.kind.is_ram() {
                if last.range.end() == region.range.start() {
                    last.range = last.range.extend_len(region.range.len());
                    return;
                }
            }
        }

        if self.allocatable.1 < MAX_ALLOCATABLE_REGIONS {
            self.allocatable.0[self.allocatable.1] = region;
            self.allocatable.1 += 1;
        }
    }

    fn insert_device(&mut self, region: MemoryRegion) -> bool {
        if region.range.is_empty() || !region.kind.is_device() {
            return false;
        }

        if self.devices().any(|existing| {
            region.range.overlaps(existing.range) && !region.range.is_adjacent_to(existing.range)
        }) {
            return false;
        }

        for i in 0..self.allocatable.1 {
            let existing = self.allocatable.0[i];
            if !existing.kind.is_device() {
                continue;
            }
            if region.range.start() == existing.range.end() {
                self.allocatable.0[i].range = existing.range.extend_len(region.range.len());
                return true;
            }
            if existing.range.start() == region.range.end() {
                self.allocatable.0[i].range = region.range.extend_len(existing.range.len());
                return true;
            }
        }

        if self.allocatable.1 >= MAX_ALLOCATABLE_REGIONS {
            return false;
        }

        self.allocatable.0[self.allocatable.1] = region;
        self.allocatable.1 += 1;
        true
    }

    pub fn debug_allocatable(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.ram() {
            f(&format_args!(
                "{:016X}..={:016X} ({:?})",
                region.range.start().as_u64(),
                region.range.end_inclusive().as_u64(),
                region.kind,
            ));
        }
    }

    pub fn debug_device(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.devices() {
            f(&format_args!(
                "{:016X}..={:016X} ({:?})",
                region.range.start().as_u64(),
                region.range.end_inclusive().as_u64(),
                region.kind,
            ));
        }
    }

    pub fn debug_io_mapped(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.io_mapped() {
            f(&format_args!(
                "{:016X}..={:016X} ({:?})",
                region.range.start().as_u64(),
                region.range.end_inclusive().as_u64(),
                region.kind,
            ));
        }
    }

    pub fn debug_snapshot(&self, f: impl Fn(&Arguments<'_>)) {
        for i in 0..self.snapshot.1 {
            let region = self.snapshot.0[i];
            f(&format_args!(
                "{:016X}..={:016X} ({:?})",
                region.range.start().as_u64(),
                region.range.end_inclusive().as_u64(),
                region.kind,
            ));
        }
    }
}

impl MemoryMap {
    pub fn dump(&self) {
        log::debug!("Memory map:");
        log::debug!("Allocatable:");
        self.debug_allocatable(|args| log::debug!("{args}"));
        log::debug!("Devices:");
        self.debug_device(|args| log::debug!("{args}"));
        log::debug!("IO mapped:");
        self.debug_io_mapped(|args| log::debug!("{args}"));
        log::debug!("Snapshot:");
        self.debug_snapshot(|args| log::debug!("{args}"));
    }
}

impl fmt::Display for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
