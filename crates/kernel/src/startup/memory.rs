use spin::Mutex;

use crate::boot::memory::{BootMemoryMap, BootMemoryRegion, ReclaimStage};
use crate::definitions::PhysicalAddress;
use crate::kernel::memory::{MemoryMap, MemoryRegion, MAX_SNAPSHOT_REGIONS};

static MEMORY: Mutex<MemoryMap> = Mutex::new(MemoryMap::empty());

/// Build the global [`MemoryMap`] from any bootloader map (no heap).
pub fn init_from_boot<M: BootMemoryMap>(boot: &M) {
    let mut snapshot = [MemoryRegion::EMPTY; MAX_SNAPSHOT_REGIONS];
    let count = boot.region_count().min(MAX_SNAPSHOT_REGIONS);

    for i in 0..count {
        let Some(region) = boot.region(i) else {
            break;
        };
        snapshot[i] = region.to_memory_region();
    }

    *MEMORY.lock() = MemoryMap::from_snapshot(snapshot, count);
}

pub fn with_memory<R>(f: impl FnOnce(&MemoryMap) -> R) -> R {
    let guard = MEMORY.lock();
    f(&guard)
}

pub fn with_memory_mut<R>(f: impl FnOnce(&mut MemoryMap) -> R) -> R {
    let mut guard = MEMORY.lock();
    f(&mut *guard)
}

/// Register a physical MMIO range (e.g. from ACPI or a driver).
pub fn register_device_memory(base: PhysicalAddress, length: usize) -> bool {
    with_memory_mut(|map| map.register_device(base, length))
}

/// Advance the allocatable pool through `stage` (e.g. after ACPI parse).
pub fn promote_through(stage: ReclaimStage) {
    with_memory_mut(|map| map.promote_through(stage));
}
