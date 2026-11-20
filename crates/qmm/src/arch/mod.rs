use core::ptr;

#[cfg(all(
    target_pointer_width = "64",
    any(target_arch = "riscv64", feature = "host-tests")
))]
pub mod riscv64;

#[cfg(all(feature = "std", target_pointer_width = "64"))]
pub mod emulator;

#[cfg(all(feature = "std", target_pointer_width = "64"))]
pub use emulator::EmulateArch;

use crate::{PhysicalAddress, TableKind, VirtAddrNotValid, VirtualAddress};

pub trait Arch: Clone + Copy {
    /// Whether the kernel has a separate page table.
    ///
    /// If true, the kernel has a separate page table from the user page table.
    /// If false, the kernel and user share the same page table, and the entries
    /// corresponding to the top half of the address space are copied into every
    /// page table and are never unmapped when unmapping user pages.
    const KERNEL_SEPARATE_TABLE: bool;

    // Page size and page table levels

    /// Page shift — how many bits to shift the page address to get the page number.
    const PAGE_SHIFT: usize;
    /// Page entry shift — how many bits index one level of the page table.
    const PAGE_ENTRY_SHIFT: usize;
    /// Number of levels of page tables for this architecture.
    const PAGE_LEVELS: usize;

    // Entry address width and shift

    /// Number of bits of physical address stored in a PTE.
    const ENTRY_ADDRESS_WIDTH: usize;
    /// Offset of the physical address field inside a PTE.
    const ENTRY_ADDRESS_SHIFT: usize = Self::PAGE_SHIFT;

    // Entry flags

    const ENTRY_FLAG_DEFAULT_PAGE: usize;
    const ENTRY_FLAG_DEFAULT_TABLE: usize;
    const ENTRY_FLAG_PRESENT: usize;
    const ENTRY_FLAG_READONLY: usize;
    const ENTRY_FLAG_READWRITE: usize;
    const ENTRY_FLAG_PAGE_USER: usize; // Leaf table user page flag
    const ENTRY_FLAG_TABLE_USER: usize = Self::ENTRY_FLAG_PAGE_USER; // Directory user page table flag
    const ENTRY_FLAG_NO_EXEC: usize;
    const ENTRY_FLAG_EXEC: usize;
    const ENTRY_FLAG_GLOBAL: usize;
    const ENTRY_FLAG_NO_GLOBAL: usize;
    /// Mask of R/W/X permission bits; non-zero means a leaf PTE (not a next-level pointer).
    const ENTRY_LEAF_PERM_MASK: usize;
    /// Mask of PTE bits used for PBMT / page memory-type encoding. Zero when absent.
    const ENTRY_PBMT_MASK: usize;
    const ENTRY_FLAG_DEVICE_MEMORY: usize;
    const ENTRY_FLAG_UNCACHEABLE: usize;
    const ENTRY_FLAG_WRITE_COMBINING: usize;

    /// Typical HHDM offset when no bootloader value has been registered yet.
    ///
    /// Real kernels should call [`crate::set_hhdm_offset`] during early boot.
    const DEFAULT_HHDM_OFFSET: usize;

    const PAGE_SIZE: usize = 1 << Self::PAGE_SHIFT;
    const PAGE_OFFSET_MASK: usize = Self::PAGE_SIZE - 1;
    const PAGE_ADDRESS_SHIFT: usize =
        Self::PAGE_LEVELS * Self::PAGE_ENTRY_SHIFT + Self::PAGE_SHIFT;
    const PAGE_ADDRESS_SIZE: u64 = 1 << (Self::PAGE_ADDRESS_SHIFT as u64);
    const PAGE_ADDRESS_MASK: usize =
        (Self::PAGE_ADDRESS_SIZE - (Self::PAGE_SIZE as u64)) as usize;
    const PAGE_ENTRY_SIZE: usize = 1 << (Self::PAGE_SHIFT - Self::PAGE_ENTRY_SHIFT);
    const PAGE_ENTRIES: usize = 1 << Self::PAGE_ENTRY_SHIFT;
    const PAGE_ENTRY_MASK: usize = Self::PAGE_ENTRIES - 1;
    const PAGE_NEGATIVE_MASK: usize = !(Self::PAGE_ADDRESS_SIZE - 1) as usize;

    const ENTRY_ADDRESS_SIZE: usize = 1 << Self::ENTRY_ADDRESS_WIDTH;
    const ENTRY_ADDRESS_MASK: usize = Self::ENTRY_ADDRESS_SIZE - 1;
    const ENTRY_FLAGS_MASK: usize =
        !(Self::ENTRY_ADDRESS_MASK << Self::ENTRY_ADDRESS_SHIFT);

    /// Sign-extends `addr` using this architecture's virtual address width.
    #[inline]
    fn sign_extend_virt(addr: usize) -> usize {
        crate::sign_extend_virt(addr, Self::PAGE_ADDRESS_SHIFT as u32)
    }

    /// Returns whether `addr` is canonical for this architecture.
    #[inline]
    fn is_canonical_virt(addr: usize) -> bool {
        crate::is_canonical_virt(addr, Self::PAGE_ADDRESS_SHIFT as u32)
    }

    /// Sign-extends `addr` to canonical and wraps it as a [`VirtualAddress`].
    #[inline]
    fn canonicalize_virt(addr: usize) -> VirtualAddress {
        unsafe { VirtualAddress::new_unchecked(Self::sign_extend_virt(addr)) }
    }

    /// Tries to wrap `addr` as a canonical [`VirtualAddress`].
    #[inline]
    fn try_virt(addr: usize) -> Result<VirtualAddress, VirtAddrNotValid> {
        if Self::is_canonical_virt(addr) {
            Ok(unsafe { VirtualAddress::new_unchecked(addr) })
        } else {
            Err(VirtAddrNotValid(addr))
        }
    }

    /// Whether `address` is a valid canonical virtual address for this architecture.
    #[inline]
    fn virt_is_valid(address: VirtualAddress) -> bool {
        Self::is_canonical_virt(address.address())
    }

    /// Read a value from a virtual address.
    ///
    /// # Safety
    ///
    /// The address must be valid, mapped, and correctly aligned for `T`.
    #[inline(always)]
    unsafe fn read<T>(address: VirtualAddress) -> T {
        unsafe { ptr::read(address.address() as *const T) }
    }

    /// Write a value to a virtual address.
    ///
    /// # Safety
    ///
    /// The address must be valid, mapped, writable, and correctly aligned for `T`.
    #[inline(always)]
    unsafe fn write<T>(address: VirtualAddress, value: T) {
        unsafe { ptr::write(address.address() as *mut T, value) }
    }

    /// Fill a region of virtual memory with a byte value.
    ///
    /// # Safety
    ///
    /// The region must be valid and mapped for writes.
    #[inline(always)]
    unsafe fn write_bytes(address: VirtualAddress, value: u8, count: usize) {
        unsafe { ptr::write_bytes(address.address() as *mut u8, value, count) }
    }

    /// Invalidate a cached translation for one virtual address.
    fn invalidate(address: VirtualAddress);

    /// Invalidate all cached translations on this CPU.
    fn invalidate_all();

    /// Return the physical address of the active page table for `table_kind`.
    fn table(table_kind: TableKind) -> PhysicalAddress;

    /// Install `address` as the active page table for `table_kind`.
    ///
    /// # Safety
    ///
    /// `address` must point to a valid, correctly structured page table.
    unsafe fn set_table(table_kind: TableKind, address: PhysicalAddress);

    /// Active HHDM offset: bootloader value if set, otherwise [`Self::DEFAULT_HHDM_OFFSET`].
    #[inline(always)]
    fn hhdm_offset() -> usize {
        crate::hhdm::hhdm_offset_or(Self::DEFAULT_HHDM_OFFSET)
    }

    /// Convert a physical address to its HHDM / direct-map virtual address.
    #[inline(always)]
    fn phys_to_virt(phys: PhysicalAddress) -> VirtualAddress {
        let addr = phys
            .address()
            .checked_add(Self::hhdm_offset())
            .unwrap_or_else(|| panic!("phys_to_virt({:#x}) overflow during HHDM offset calculation", phys.address()));
        Self::try_virt(addr)
            .unwrap_or_else(|_| panic!("phys_to_virt({:#x}) produced non-canonical address during HHDM offset calculation", addr))
    }
}
