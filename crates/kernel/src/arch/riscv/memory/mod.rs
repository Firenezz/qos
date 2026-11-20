use qmm::{TableKind, arch::Arch, page::structures::table::PageTable};

use crate::definitions::PhysicalAddress;
use crate::definitions::VirtualAddress;

pub type PageTableEntry = qmm::page::structures::entry::PageTableEntry<CurrentRmmArch>;

use crate::arch::{riscv::Riscv64, traits::Memory};
use crate::kernel::memory::MemoryMap;

pub use qmm::arch::riscv64::Sv48Arch as CurrentRmmArch;

static mut HHDM_OFFSET: u64 = 0;

impl Memory for Riscv64 {
    fn init_memory(info: &crate::boot::BootInfo) {
        let hhdm_offset = info.hhdm_offset.unwrap_or(0) as usize;
        qmm::set_hhdm_offset(hhdm_offset);

        let arch_hhdm_offset = CurrentRmmArch::hhdm_offset();
        
        log::info!("Arch HHDM offset: {:X}", arch_hhdm_offset as u64);

        let root = CurrentRmmArch::table(TableKind::Kernel);

        let root_table = PageTable::<CurrentRmmArch>::from_physical(root);

        let kernel_page = unsafe { root_table.entry(511) }.and_then(|entry| entry.as_table()).expect("Kernel page table entry not found");

        log::info!("Root table: {:X}", root_table.physical_address().address() as u64);

        kernel_page.debug_tree_special(|arg| log::info!("{}", arg));

        let memory_map = MemoryMap::new();

        

    }

    fn get_highest_table_level() -> usize {
        4
    }

    fn get_highest_table_physical_address() -> PhysicalAddress {
        todo!("Not implemented")
    }

    fn get_highest_table_virtual_address() -> VirtualAddress {
        todo!()
    }
    
    fn set_hhdm_offset(offset: u64) {
        unsafe {
            HHDM_OFFSET = offset;
        }
    }
    
    fn get_hhdm_offset() -> u64 {
        unsafe {
            HHDM_OFFSET
        }
    }
    
    fn get_hhdm_mapped_physical_address(physical_address: PhysicalAddress) -> VirtualAddress {
        VirtualAddress::new((Self::get_hhdm_offset() + physical_address.address() as u64) as usize)
    }
}

impl Riscv64 {
    pub fn find_entry_with_physical_address(physical_address: PhysicalAddress) -> Option<PageTableEntry> {
        todo!("Not implemented")
    }
}