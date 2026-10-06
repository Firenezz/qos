use crate::PhysicalAddress;

pub mod bump;

#[derive(Debug, Clone, Copy)]
pub struct FrameCount(usize);

impl FrameCount {
    pub fn new(count: usize) -> Self {
        Self(count)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FrameUsage {
    pub total: FrameCount,
    pub used: FrameCount,
}

impl FrameUsage {
    pub fn new(used: FrameCount, total: FrameCount) -> Self {
        Self { used, total }
    }

    pub fn used(&self) -> FrameCount {
        self.used
    }

    pub fn total(&self) -> FrameCount {
        self.total
    }

    pub fn is_full(&self) -> bool {
        self.used.0 == self.total.0
    }

    pub fn is_empty(&self) -> bool {
        self.used.0 == 0
    }

    pub fn free(&self) -> FrameCount {
        FrameCount(self.total.0 - self.used.0)
    }
}

pub unsafe trait FrameAllocator {
    fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress>;

    unsafe fn deallocate(&mut self, address: PhysicalAddress, count: FrameCount);

    fn allocate_one(&mut self) -> Option<PhysicalAddress> {
        self.allocate(FrameCount::new(1))
    }

    unsafe fn deallocate_one(&mut self, address: PhysicalAddress) {
        unsafe {
            self.deallocate(address, FrameCount::new(1));
        }
    }

    fn usage(&self) -> FrameUsage;
}

unsafe impl<T> FrameAllocator for &mut T
where
    T: FrameAllocator,
{
    fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        T::allocate(self, count)
    }
    unsafe fn deallocate(&mut self, address: PhysicalAddress, count: FrameCount) {
        unsafe { T::deallocate(self, address, count) }
    }
    fn allocate_one(&mut self) -> Option<PhysicalAddress> {
        T::allocate_one(self)
    }
    unsafe fn deallocate_one(&mut self, address: PhysicalAddress) {
        unsafe { T::deallocate_one(self, address) }
    }
    fn usage(&self) -> FrameUsage {
        T::usage(self)
    }
}
