use core::{fmt, marker::PhantomData};

use crate::{PhysicalAddress, VirtualAddress, arch::Arch};

use super::entry::PageTableEntry;

pub struct PageTable<A> {
    base_address: VirtualAddress,
    physical_address: PhysicalAddress,
    level: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageTable<A> {
    pub unsafe fn new(
        base_address: VirtualAddress,
        physical_address: PhysicalAddress,
        level: usize,
    ) -> Self {
        Self {
            base_address,
            physical_address,
            level,
            phantom: PhantomData,
        }
    }

    /// View the active root page table.
    ///
    /// `physical_address` is where the table lives in RAM (via HHDM when reading
    /// entries). `base_address` is the virtual-address origin this table indexes
    /// (zero for the root table).
    pub fn from_physical(physical_address: PhysicalAddress) -> Self {
        Self {
            base_address: VirtualAddress::zero(),
            physical_address,
            level: A::PAGE_LEVELS - 1,
            phantom: PhantomData,
        }
    }

    pub fn base_address(&self) -> VirtualAddress {
        self.base_address
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        self.physical_address
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn entry_base_address(&self, i: usize) -> Option<VirtualAddress> {
        if i < A::PAGE_ENTRIES {
            let level_shift = self.level * A::PAGE_ENTRY_SHIFT + A::PAGE_SHIFT;
            Some(self.base_address.add(i << level_shift))
        } else {
            None
        }
    }

    unsafe fn entry_virt_address(&self, i: usize) -> Option<VirtualAddress> {
        if i < A::PAGE_ENTRIES {
            Some(A::phys_to_virt(self.physical_address).add(i * A::PAGE_ENTRY_SIZE))
        } else {
            None
        }
    }

    pub unsafe fn entry(&self, i: usize) -> Option<PageTableEntry<A>> {
        unsafe {
            let addr = self.entry_virt_address(i)?;
            Some(PageTableEntry::from_data(A::read::<usize>(addr)))
        }
    }

    pub(crate) unsafe fn set_entry(&mut self, i: usize, entry: PageTableEntry<A>) -> Option<()> {
        unsafe {
            let addr = self.entry_virt_address(i)?;
            A::write::<usize>(addr, entry.data());
            Some(())
        }
    }

    pub fn index_of(&self, address: VirtualAddress) -> Option<usize> {
        let address = VirtualAddress::new(address.as_usize() & A::PAGE_ADDRESS_MASK);
        let level_shift = self.level * A::PAGE_ENTRY_SHIFT + A::PAGE_SHIFT;
        let level_mask = A::PAGE_ENTRIES
            .wrapping_shl(level_shift as u32)
            .wrapping_sub(1);
        if address >= self.base_address && address <= self.base_address.add(level_mask) {
            Some((address.as_usize() >> level_shift) & A::PAGE_ENTRY_MASK)
        } else {
            None
        }
    }

    pub unsafe fn next_level_table(&self, i: usize) -> Option<Self> {
        if self.level == 0 {
            return None;
        }

        unsafe {
            let entry = self.entry(i)?;
            if !entry.is_table_pointer() {
                return None;
            }
            Some(PageTable::new(
                self.entry_base_address(i)?,
                entry.address().ok()?,
                self.level - 1,
            ))
        }
    }

    pub fn debug_entries(&self, f: impl Fn(fmt::Arguments<'_>)) {
        for i in 0..A::PAGE_ENTRIES {
            if let Some(entry) = unsafe { self.entry(i) }
                && entry.present()
            {
                f(format_args!("{}: 0x{:X}", i, entry.data()));
            }
        }
    }

    pub fn debug_entries_with_address(&self, f: impl Fn(fmt::Arguments<'_>)) {
        for i in 0..A::PAGE_ENTRIES {
            if let Some(entry) = unsafe { self.entry(i) }
                && entry.present()
            {
                f(format_args!(
                    "{}: 0x{:X} -> 0x{:X}",
                    i,
                    entry.data(),
                    A::phys_to_virt(PhysicalAddress::new(entry.data())).address()
                ));
            }
        }
    }

    pub fn debug_entries_with_address_and_flags(&self, f: impl Fn(fmt::Arguments<'_>)) {
        for i in 0..A::PAGE_ENTRIES {
            if let Some(entry) = unsafe { self.entry(i) }
                && entry.present()
            {
                f(format_args!(
                    "{}: 0x{:X} -> 0x{:X} \n flags: {:?}",
                    i,
                    entry.data(),
                    A::phys_to_virt(PhysicalAddress::new(entry.data())).address(),
                    entry.flags()
                ));
            }
        }
    }
}
