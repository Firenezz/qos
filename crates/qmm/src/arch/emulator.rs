//! Software-emulated architecture for host-side tests.
//!
//! Uses an Sv48-shaped address space with a linear HHDM direct map:
//! `virt = hhdm_offset + phys`.

extern crate std;

use core::ptr;
use std::sync::Mutex;

use crate::{MEGABYTE, MemoryArea, PhysicalAddress, TableKind, VirtualAddress};

use super::Arch;

const MEMORY_SIZE: usize = 64 * MEGABYTE;

static MEMORY_AREAS: [MemoryArea; 1] = [MemoryArea {
    base: PhysicalAddress(0),
    size: MEMORY_SIZE,
}];

static MACHINE: Mutex<Option<Machine>> = Mutex::new(None);

#[derive(Clone, Copy)]
pub struct EmulateArch;

impl EmulateArch {
    /// Initialize the emulated machine and return available physical memory.
    pub fn init() -> &'static [MemoryArea] {
        crate::hhdm::set_hhdm_offset(Self::DEFAULT_HHDM_OFFSET);
        *MACHINE.lock().unwrap() = Some(Machine::new(MEMORY_SIZE));
        &MEMORY_AREAS
    }

    fn with_machine<F, R>(f: F) -> R
    where
        F: FnOnce(&Machine) -> R,
    {
        let guard = MACHINE.lock().unwrap();
        f(guard.as_ref().expect("EmulateArch::init was not called"))
    }

    fn with_machine_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Machine) -> R,
    {
        let mut guard = MACHINE.lock().unwrap();
        f(guard.as_mut().expect("EmulateArch::init was not called"))
    }
}

impl Arch for EmulateArch {
    const KERNEL_SEPARATE_TABLE: bool = false;

    const PAGE_SHIFT: usize = 12;
    const PAGE_ENTRY_SHIFT: usize = 9;
    const PAGE_LEVELS: usize = 4;

    const ENTRY_ADDRESS_WIDTH: usize = 44;
    const ENTRY_ADDRESS_SHIFT: usize = 10;

    const ENTRY_FLAG_DEFAULT_PAGE: usize = Self::ENTRY_FLAG_PRESENT | Self::ENTRY_FLAG_READONLY;
    const ENTRY_FLAG_DEFAULT_TABLE: usize = Self::ENTRY_FLAG_PRESENT;
    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_READONLY: usize = 1 << 1;
    const ENTRY_FLAG_READWRITE: usize = 3 << 1;
    const ENTRY_FLAG_PAGE_USER: usize = 1 << 4;
    const ENTRY_FLAG_NO_EXEC: usize = 0;
    const ENTRY_FLAG_EXEC: usize = 1 << 3;
    const ENTRY_FLAG_GLOBAL: usize = 1 << 5;
    const ENTRY_FLAG_NO_GLOBAL: usize = 0;
    const ENTRY_LEAF_PERM_MASK: usize = (1 << 1) | (1 << 2) | (1 << 3);
    const ENTRY_PBMT_MASK: usize = 0;
    const ENTRY_FLAG_DEVICE_MEMORY: usize = 0;
    const ENTRY_FLAG_UNCACHEABLE: usize = 0;
    const ENTRY_FLAG_WRITE_COMBINING: usize = 0;

    const DEFAULT_HHDM_OFFSET: usize = 0xFFFF_8000_0000_0000;

    unsafe fn read<T>(address: VirtualAddress) -> T {
        EmulateArch::with_machine(|machine| machine.read(address))
    }

    unsafe fn write<T>(address: VirtualAddress, value: T) {
        EmulateArch::with_machine_mut(|machine| machine.write(address, value));
    }

    unsafe fn write_bytes(address: VirtualAddress, value: u8, count: usize) {
        EmulateArch::with_machine_mut(|machine| machine.write_bytes(address, value, count));
    }

    fn invalidate(_address: VirtualAddress) {}

    fn invalidate_all() {}

    fn table(_table_kind: TableKind) -> PhysicalAddress {
        EmulateArch::with_machine(|machine| machine.table())
    }

    unsafe fn set_table(_table_kind: TableKind, address: PhysicalAddress) {
        EmulateArch::with_machine_mut(|machine| machine.set_table(address));
    }
}

struct Machine {
    memory: Box<[u8]>,
    table: PhysicalAddress,
}

impl Machine {
    fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size].into_boxed_slice(),
            table: PhysicalAddress::new(0),
        }
    }

    fn translate(&self, virt: VirtualAddress) -> Option<PhysicalAddress> {
        let virt_addr = virt.address();
        let phys_addr = virt_addr.checked_sub(EmulateArch::hhdm_offset())?;
        if phys_addr < self.memory.len() {
            Some(PhysicalAddress::new(phys_addr))
        } else {
            None
        }
    }

    fn read<T>(&self, virt: VirtualAddress) -> T {
        let phys = self
            .translate(virt)
            .unwrap_or_else(|| panic!("read: {virt:#x?} not mapped"));
        let offset = phys.address();
        let size = core::mem::size_of::<T>();
        if offset + size > self.memory.len() {
            panic!("read: {virt:#x?} size {size:#x} out of bounds");
        }
        // SAFETY: bounds checked above; machine memory is valid for the object's lifetime.
        unsafe { ptr::read(self.memory.as_ptr().add(offset) as *const T) }
    }

    fn write<T>(&mut self, virt: VirtualAddress, value: T) {
        let phys = self
            .translate(virt)
            .unwrap_or_else(|| panic!("write: {virt:#x?} not mapped"));
        let offset = phys.address();
        let size = core::mem::size_of::<T>();
        if offset + size > self.memory.len() {
            panic!("write: {virt:#x?} size {size:#x} out of bounds");
        }
        // SAFETY: bounds checked above; machine memory is valid for the object's lifetime.
        unsafe { ptr::write(self.memory.as_mut_ptr().add(offset) as *mut T, value) }
    }

    fn write_bytes(&mut self, virt: VirtualAddress, value: u8, count: usize) {
        let phys = self
            .translate(virt)
            .unwrap_or_else(|| panic!("write_bytes: {virt:#x?} not mapped"));
        let offset = phys.address();
        if offset + count > self.memory.len() {
            panic!("write_bytes: {virt:#x?} count {count:#x} out of bounds");
        }
        // SAFETY: bounds checked above; machine memory is valid for the object's lifetime.
        unsafe {
            ptr::write_bytes(self.memory.as_mut_ptr().add(offset), value, count);
        }
    }

    fn table(&self) -> PhysicalAddress {
        self.table
    }

    fn set_table(&mut self, address: PhysicalAddress) {
        self.table = address;
    }
}
