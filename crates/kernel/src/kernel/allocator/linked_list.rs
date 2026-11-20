use core::alloc::{GlobalAlloc, Layout};

use linked_list_allocator::Heap;
use spin::Mutex;

static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub struct Allocator;

impl Allocator {
    pub unsafe fn init (offset: usize, size: usize) {
        unsafe { *HEAP.lock() = Some(Heap::new(offset as *mut u8, size)); }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = HEAP.lock();
        unimplemented!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            unimplemented!()
        }
    }
}