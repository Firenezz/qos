use core::fmt::{self, Arguments};

use crate::boot::memory::ReclaimStage;
use crate::definitions::PhysicalAddress;
use crate::definitions::PhysicalAddressRange;

pub mod frame;
pub mod region;

pub const MAX_SNAPSHOT_REGIONS: usize = 128;
pub const MAX_ALLOCATABLE_REGIONS: usize = 64;

/// Immutable bootloader memmap snapshot plus the kernel's managed region pool.
#[derive(Clone)]
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
}

impl fmt::Display for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryMap {{")?;
        write!(f, "snapshot: {:?}", self.snapshot)?;
        write!(f, "allocatable: {:?}", self.allocatable)?;
        write!(f, "reclaim_stage: {:?}", self.reclaim_stage)?;
        write!(f, "}}")
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryMap {{")?;
        write!(f, "snapshot: {:?}", self.snapshot)?;
        write!(f, "allocatable: {:?}", self.allocatable)?;
        write!(f, "reclaim_stage: {:?}", self.reclaim_stage)?;
        write!(f, "}}")
    }
}

impl MemoryMap {
    pub fn debug_allocatable(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.ram() {
            f(&format_args!(
                "{}..={} ({:?})",
                region.range.start(),
                region.range.end_inclusive(),
                region.kind,
            ));
        }
    }

    pub fn debug_device(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.devices() {
            f(&format_args!(
                "{}..={} ({:?})",
                region.range.start(),
                region.range.end_inclusive(),
                region.kind,
            ));
        }
    }

    pub fn debug_io_mapped(&self, f: impl Fn(&Arguments<'_>)) {
        for region in self.io_mapped() {
            f(&format_args!(
                "{}..={} ({:?})",
                region.range.start(),
                region.range.end_inclusive(),
                region.kind,
            ));
        }
    }

    pub fn debug_snapshot(&self, f: impl Fn(&Arguments<'_>)) {
        for i in 0..self.snapshot.1 {
            let region = self.snapshot.0[i];
            f(&format_args!(
                "{}..={} ({:?})",
                region.range.start(),
                region.range.end_inclusive(),
                region.kind,
            ));
        }
    }
}

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
    /// ACPI/DT explicitly registers MMIO via [`MemoryMap::register_device`].
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
