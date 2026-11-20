//! Physical page-frame allocator.
//!
//! Usable RAM from the boot memory map is page-aligned into a static area table
//! and handed to [`qmm`]'s bump allocator. Frames are allocated in order and
//! zeroed through the higher-half direct map. Freeing is not supported.

use core::cell::UnsafeCell;

use spin::Mutex;

use qmm::allocator::bump::BumpAllocator;
use qmm::allocator::{FrameAllocator, FrameCount, FrameUsage};
use qmm::arch::Arch;
use qmm::arch::riscv64::Sv48Arch;
use qmm::structures::{MemoryArea, PhysicalAddress};
use qmm::{align_down_u64, align_up_u64};

use super::{MAX_ALLOCATABLE_REGIONS, MemoryMap, MemoryRegion};

type RmmArch = Sv48Arch;

const MAX_AREAS: usize = MAX_ALLOCATABLE_REGIONS;

const EMPTY_AREA: MemoryArea = MemoryArea {
    base: PhysicalAddress(0),
    size: 0,
};

struct AreaStore {
    areas: [MemoryArea; MAX_AREAS],
    len: usize,
}

/// Area table shared with the bump allocator after init.
///
/// # Safety
///
/// Mutated only from [`fill_areas`] on the bootstrap hart, before the resulting
/// slice is stored in the frame allocator and before other harts run.
struct AreaTable(UnsafeCell<AreaStore>);

// SAFETY: see [`AreaTable`]. The store is written once during BSP init, then only read.
unsafe impl Sync for AreaTable {}

static AREAS: AreaTable = AreaTable(UnsafeCell::new(AreaStore {
    areas: [EMPTY_AREA; MAX_AREAS],
    len: 0,
}));

static FRAMES: Mutex<Option<PhysicalMemoryManager<RmmArch>>> = Mutex::new(None);

/// Bump allocator over the physical RAM published by [`MemoryMap`].
pub struct PhysicalMemoryManager<A> {
    bump: BumpAllocator<A>,
}

impl<A: Arch> PhysicalMemoryManager<A> {
    /// Build an allocator from `map`'s RAM regions.
    ///
    /// # Safety
    ///
    /// * Called once on the bootstrap hart, before any other use of the area table.
    /// * [`qmm`] HHDM is installed, so allocated frames can be zeroed.
    /// * `map`'s RAM regions are free: they do not cover the kernel image or live firmware.
    pub unsafe fn from_map(map: &MemoryMap) -> Self {
        let areas = fill_areas(map, A::PAGE_SIZE);
        Self {
            // SAFETY: `areas` is the static area table, immutable after this fill.
            bump: unsafe { BumpAllocator::new((areas, 0), (areas, 0)) },
        }
    }

    pub fn allocate_one(&mut self) -> Option<PhysicalAddress> {
        FrameAllocator::allocate_one(self)
    }

    pub fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        FrameAllocator::allocate(self, count)
    }

    /// # Safety
    ///
    /// `address` must be a block previously returned by [`Self::allocate`] for `count` frames,
    /// and must not be used after this call. The bump allocator does not reclaim frames.
    pub unsafe fn deallocate(&mut self, address: PhysicalAddress, count: FrameCount) {
        unsafe { FrameAllocator::deallocate(self, address, count) }
    }

    pub fn usage(&self) -> FrameUsage {
        FrameAllocator::usage(self)
    }
}

unsafe impl<A: Arch> FrameAllocator for PhysicalMemoryManager<A> {
    fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        self.bump.allocate(count)
    }

    unsafe fn deallocate(&mut self, address: PhysicalAddress, count: FrameCount) {
        unsafe { self.bump.deallocate(address, count) }
    }

    fn usage(&self) -> FrameUsage {
        self.bump.usage()
    }
}

/// Install the global frame allocator from the current memory map.
///
/// # Safety
///
/// See [`PhysicalMemoryManager::from_map`]. Must run after the qmm HHDM offset is set.
pub unsafe fn init(map: &MemoryMap) {
    let mut slot = FRAMES.lock();
    if slot.is_some() {
        log::warn!("page frame allocator already initialized");
        return;
    }

    let manager = unsafe { PhysicalMemoryManager::<RmmArch>::from_map(map) };
    let usage = manager.usage();
    log::info!(
        "page frame allocator: {} free / {} total frames ({} bytes)",
        usage.free().get(),
        usage.total().get(),
        usage.total().get().saturating_mul(RmmArch::PAGE_SIZE),
    );
    *slot = Some(manager);
}

/// Run `f` with the global frame allocator, if [`init`] has completed.
///
/// The allocator lock is held for the duration of `f`.
pub fn with_frame_allocator<R>(
    f: impl FnOnce(&mut PhysicalMemoryManager<RmmArch>) -> R,
) -> Option<R> {
    FRAMES.lock().as_mut().map(f)
}

/// Allocate `count` physically contiguous frames. `None` if the allocator is not
/// installed or the request cannot be satisfied.
pub fn allocate(count: usize) -> Option<PhysicalAddress> {
    with_frame_allocator(|manager| manager.allocate(FrameCount::new(count))).flatten()
}

/// Allocate one physical frame.
pub fn allocate_one() -> Option<PhysicalAddress> {
    allocate(1)
}

fn fill_areas(map: &MemoryMap, page_size: usize) -> &'static [MemoryArea] {
    // SAFETY: sole caller is `from_map`, which runs once on the BSP before the
    // slice is shared with the bump allocator. The store is not mutated after return.
    let store = unsafe { &mut *AREAS.0.get() };
    store.len = 0;

    for region in map.ram() {
        if store.len >= MAX_AREAS {
            log::warn!("page frame allocator: area table full, remaining RAM ignored");
            break;
        }
        let Some(area) = aligned_area(region, page_size) else {
            continue;
        };
        store.areas[store.len] = area;
        store.len += 1;
        log::debug!(
            "frame area {:#x} size {:#x}",
            area.base().as_u64(),
            area.size
        );
    }

    let len = store.len;
    unsafe { core::slice::from_raw_parts(store.areas.as_ptr(), len) }
}

fn aligned_area(region: &MemoryRegion, page_size: usize) -> Option<MemoryArea> {
    let page_size = page_size as u64;
    let start = align_up_u64(region.range.start().as_u64(), page_size);
    let end = align_down_u64(region.range.end().as_u64(), page_size);
    let size = end.saturating_sub(start);
    if size < page_size {
        return None;
    }
    Some(MemoryArea::new(
        PhysicalAddress::new(start as usize),
        size as usize,
    ))
}
