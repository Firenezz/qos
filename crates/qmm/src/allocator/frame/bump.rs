use core::marker::PhantomData;

use crate::{
    MemoryArea, PhysicalAddress,
    allocator::{FrameAllocator, FrameCount, FrameUsage},
    arch::Arch,
};

pub struct BumpAllocator<A> {
    orig_areas: (&'static [MemoryArea], usize),
    cur_areas: (&'static [MemoryArea], usize),
    _marker: PhantomData<fn() -> A>,
}

impl<A: Arch> BumpAllocator<A> {
    pub unsafe fn new(
        orig_areas: (&'static [MemoryArea], usize),
        cur_areas: (&'static [MemoryArea], usize),
    ) -> Self {
        Self {
            orig_areas,
            cur_areas,
            _marker: PhantomData,
        }
    }

    pub fn areas(&self) -> (&'static [MemoryArea], &'static [MemoryArea]) {
        (self.orig_areas.0, self.cur_areas.0)
    }

    pub fn free_areas(&self) -> (&'static [MemoryArea], usize) {
        (self.orig_areas.0, self.orig_areas.1)
    }

    pub fn cur_areas(&self) -> (&'static [MemoryArea], usize) {
        (self.cur_areas.0, self.cur_areas.1)
    }
    pub fn abs_offset(&self) -> PhysicalAddress {
        let (areas, off) = self.cur_areas;
        areas
            .first()
            .map_or(PhysicalAddress::new(0), |a| a.base.add(off))
    }
    pub fn offset(&self) -> usize {
        (self.usage().total().get() - self.usage().free().get()) * A::PAGE_SIZE
    }
}

unsafe impl<A: Arch> FrameAllocator for BumpAllocator<A> {
    fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        unsafe {
            let req_size = count.get() * A::PAGE_SIZE;

            let block = loop {
                let area = self.cur_areas.0.first()?;
                let off = self.cur_areas.1;
                if area.size - off < req_size {
                    self.cur_areas = (&self.cur_areas.0[1..], 0);
                    continue;
                }
                self.cur_areas.1 += req_size;

                break area.base.add(off);
            };
            A::write_bytes(A::phys_to_virt(block), 0, req_size);
            Some(block)
        }
    }

    unsafe fn deallocate(&mut self, _address: PhysicalAddress, _count: FrameCount) {
        unimplemented!("BumpAllocator::free not implemented");
    }

    fn usage(&self) -> FrameUsage {
        let total = self.orig_areas.0.iter().map(|a| a.size).sum::<usize>() - self.orig_areas.1;
        let free = self.cur_areas.0.iter().map(|a| a.size).sum::<usize>() - self.cur_areas.1;
        FrameUsage::new(
            FrameCount::new((total - free) / A::PAGE_SIZE),
            FrameCount::new(total / A::PAGE_SIZE),
        )
    }
}
