//! Early kernel heap until the frame allocator / real kheap exist.

use core::ptr::addr_of_mut;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static GLOBAL: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 0x10000000;

static mut HEAP: Heap = Heap::empty();

pub struct Heap {
    heap_storage: [u8; HEAP_SIZE],
}

impl Heap {
    pub const fn empty() -> Self {
        Self {
            heap_storage: [0; HEAP_SIZE],
        }
    }
}

/// Initialize the early heap. Must run before any `alloc` use (e.g. MP cpu table).
pub fn init() {
    // SAFETY: BSP-only; no concurrent alloc yet; HEAP is exclusive to this allocator.
    unsafe {
        GLOBAL
            .lock()
            .init(addr_of_mut!(HEAP.heap_storage).cast(), HEAP_SIZE);
    }
}
