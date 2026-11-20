//! Early kernel heap (bump) until the frame allocator / real kheap exist.

use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;

/// Bump arena size (replace with frame-backed kheap when MM is online).
const HEAP_SIZE: usize = 1024 * 1024;

#[repr(C, align(4096))]
struct HeapStorage {
    bytes: [u8; HEAP_SIZE],
}

static HEAP: HeapStorage = HeapStorage {
    bytes: [0; HEAP_SIZE],
};

struct Bump {
    start: usize,
    end: usize,
    next: usize,
}

impl Bump {
    const fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
            next: 0,
        }
    }

    fn init(&mut self) {
        self.start = HEAP.bytes.as_ptr() as usize;
        self.end = self.start + HEAP_SIZE;
        self.next = self.start;
    }

    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let align = layout.align();
        let size = layout.size();
        let aligned = (self.next + align - 1) & !(align - 1);
        let end = aligned.checked_add(size)?;
        if end > self.end {
            return None;
        }
        self.next = end;
        Some(aligned as *mut u8)
    }
}

struct BumpAllocator(Mutex<Bump>);

// SAFETY: guarded by a spin mutex.
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL: BumpAllocator = BumpAllocator(Mutex::new(Bump::empty()));

/// Initialize the bump heap. Must run before any `alloc` use (e.g. MP cpu table).
pub fn init() {
    GLOBAL.0.lock().init();
}
