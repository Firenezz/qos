use core::marker::PhantomData;

use crate::{
    PhysicalAddress,
    arch::Arch,
    page::structures::{
        flags::{PageFlags, Pbmt},
        table::PageTable,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableEntry<A> {
    data: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageTableEntry<A> {
    #[inline(always)]
    pub fn new(address: impl Into<usize>, flags: usize) -> Self {
        let data = (((address.into() >> A::PAGE_SHIFT) & A::ENTRY_ADDRESS_MASK)
            << A::ENTRY_ADDRESS_SHIFT)
            | flags;

        Self::from_data(data)
    }

    #[inline(always)]
    pub fn from_data(data: usize) -> Self {
        Self {
            data,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.data
    }

    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.present() && self.data & A::ENTRY_LEAF_PERM_MASK != 0
    }

    #[inline(always)]
    pub fn is_table_pointer(&self) -> bool {
        self.present() && self.data & A::ENTRY_LEAF_PERM_MASK == 0
    }

    /// Whether this entry is worth printing in a filtered page-table dump.
    ///
    /// Table pointers are omitted (`false`) — callers always recurse through them.
    /// Flags standard Limine kernel RAM leaves (`0xcf`-style RWX/PMA) as uninteresting.
    #[inline]
    pub fn is_noteworthy(&self, level: usize) -> bool {
        if !self.present() || self.is_table_pointer() {
            return false;
        }
        if level > 0 {
            return true;
        }
        let flags = self.flags();
        flags.has_user()
            || flags.is_global()
            || flags.pbmt_kind() != Pbmt::Pma
            || !flags.has_write()
            || !flags.has_execute()
    }

    #[inline(always)]
    pub fn flags(&self) -> PageFlags<A> {
        unsafe { PageFlags::from_data(self.data & A::ENTRY_FLAGS_MASK) }
    }

    #[inline(always)]
    pub fn address(&self) -> Result<PhysicalAddress, PhysicalAddress> {
        let addr = PhysicalAddress(
            ((self.data >> A::ENTRY_ADDRESS_SHIFT) & A::ENTRY_ADDRESS_MASK) << A::PAGE_SHIFT,
        );

        if self.present() { Ok(addr) } else { Err(addr) }
    }

    #[inline(always)]
    pub fn present(&self) -> bool {
        self.data & A::ENTRY_FLAG_PRESENT != 0
    }

    #[inline(always)]
    pub fn as_table(&self) -> Option<PageTable<A>> {
        if !self.is_table_pointer() {
            return None;
        }
        Some(PageTable::from_physical(self.address().unwrap()))
    }
}
