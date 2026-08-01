use core::marker::PhantomData;

use crate::{TableKind, allocator::FrameAllocator, arch::Arch, page::structures::{entry::PageTableEntry, table::PageTable}, structures::{PhysicalAddress, VirtualAddress}};



pub struct PageMapper<A, F> {
    table_kind: TableKind,
    table_address: PhysicalAddress,
    allocator: F,
    phantom: PhantomData<fn() -> A>,
}

impl<A: Arch, F> PageMapper<A, F> {
    unsafe fn new(table_kind: TableKind, table_address: PhysicalAddress, allocator: F) -> Self {
        Self { table_kind, table_address, allocator, phantom: PhantomData }
    }

    pub unsafe fn current(table_kind: TableKind, allocator: F) -> Self {
        let table_address = A::table(table_kind);
        unsafe { Self::new(table_kind, table_address, allocator) }
    }

    pub fn is_current(&self) -> bool {
        self.table().physical_address() == A::table(self.table_kind)
    }

    pub fn table(&self) -> PageTable<A> {
        // SAFETY: The only way to initialize a PageMapper is via new(), and we assume it upholds
        // all necessary invariants for this to be safe.
        unsafe { 
            PageTable::new(
                VirtualAddress::new(0),
                self.table_address,
                A::PAGE_LEVELS - 1,
            )
        }
    }

    pub fn allocator(&self) -> &F {
        &self.allocator
    }

    pub fn allocator_mut(&mut self) -> &mut F {
        &mut self.allocator
    }

    // TODO: PageFlags

    fn visit<T>(&self, virtual_address: VirtualAddress, visitor: impl FnOnce(PageTable<A>, usize) -> T) -> Option<T> {
        let mut table = self.table();
        loop {
            let i = table.index_of(virtual_address)?;
            if table.level() == 0 {
                return Some(visitor(table, i));
            } else {
                table = unsafe { table.next_level_table(i)? };
            }
        }
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let entry = self.visit(virtual_address, |p1, i| unsafe { p1.entry(i) })??;
        Some(entry.address().ok()?)
    }
}

impl<A: Arch, F: FrameAllocator> PageMapper<A, F> {
    pub unsafe fn create(table_kind: TableKind, mut allocator: F) -> Option<Self> {
        let table_address = allocator.allocate_one()?;
        let mut table = unsafe { Self::new(table_kind, table_address, allocator) };

        match (table_kind, A::KERNEL_SEPARATE_TABLE) {
            (TableKind::Kernel, false) => {
                for i in A::PAGE_LEVELS / 2..A::PAGE_LEVELS {
                    let phys = table
                        .allocator
                        .allocate_one()?;
                    let flags = A::ENTRY_FLAG_DEFAULT_TABLE;
                    unsafe { table
                        .table()
                        .set_entry(i, PageTableEntry::new(phys.address(), flags))? };
                }
            }
            (TableKind::User, false) => {
                let active_kernel_table = unsafe { PageMapper::current(TableKind::Kernel, ()) };
                unsafe { for i in A::PAGE_ENTRIES / 2..A::PAGE_ENTRIES {
                    if let Some(entry) = active_kernel_table.table().entry(i) {
                        table.table().set_entry(i, entry);
                    }
                } };
            }
            (_, true) => { }
        }

        Some(table)
    }
}