use crate::arch::Arch;

#[derive(Clone, Copy)]
pub struct Sv48Arch;

impl Arch for Sv48Arch {
    const KERNEL_SEPARATE_TABLE: bool = false;

    const PAGE_SHIFT: usize = 12; // 4096 bytes
    const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries, 8 bytes each
    const PAGE_LEVELS: usize = 4; // L0, L1, L2, L3

    const ENTRY_ADDRESS_WIDTH: usize = 44;
    const ENTRY_ADDRESS_SHIFT: usize = 10;

    const ENTRY_FLAG_DEFAULT_PAGE: usize = Self::ENTRY_FLAG_PRESENT | Self::ENTRY_FLAG_READONLY;
    const ENTRY_FLAG_DEFAULT_TABLE: usize = Self::ENTRY_FLAG_PRESENT;
    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_READONLY: usize = 1 << 1;
    const ENTRY_FLAG_READWRITE: usize = 3 << 1;
    const ENTRY_FLAG_PAGE_USER: usize = 1 << 4;
    const ENTRY_FLAG_TABLE_USER: usize = 0;
    const ENTRY_FLAG_NO_EXEC: usize = 0;
    const ENTRY_FLAG_EXEC: usize = 1 << 3;
    const ENTRY_FLAG_GLOBAL: usize = 1 << 5;
    const ENTRY_FLAG_NO_GLOBAL: usize = 0;
    const ENTRY_LEAF_PERM_MASK: usize = (1 << 1) | (1 << 2) | (1 << 3);

    // SVPBMT: PBMT field in PTE bits 62:61 (see RISC-V Privileged Architecture).
    const ENTRY_PBMT_MASK: usize = 0b11 << 61;
    const ENTRY_FLAG_UNCACHEABLE: usize = 1 << 61;
    const ENTRY_FLAG_WRITE_COMBINING: usize = 1 << 61;
    const ENTRY_FLAG_DEVICE_MEMORY: usize = 1 << 62;

    const DEFAULT_HHDM_OFFSET: usize = 0x0;

    fn invalidate(address: crate::VirtualAddress) {
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!(
                "sfence.vma {}",
                in(reg) address.address(),
                options(nomem, nostack, preserves_flags)
            );
        }
        #[cfg(not(target_arch = "riscv64"))]
        let _ = address;
    }

    fn invalidate_all() {
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
        }
    }

    fn table(_table_kind: crate::TableKind) -> crate::PhysicalAddress {
        #[cfg(target_arch = "riscv64")]
        {
            let local_satp = riscv_rs::csr::satp::read();
            let physical_address = local_satp.ppn() << Self::PAGE_SHIFT;
            crate::PhysicalAddress::new(physical_address as usize)
        }
        #[cfg(not(target_arch = "riscv64"))]
        crate::PhysicalAddress::new(0)
    }

    unsafe fn set_table(_table_kind: crate::TableKind, address: crate::PhysicalAddress) {
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!(
                "csrw satp, {}",
                in(reg) address.address(),
                options(nomem, nostack, preserves_flags)
            );
            Self::invalidate_all();
        }
        #[cfg(not(target_arch = "riscv64"))]
        let _ = address;
    }
}
