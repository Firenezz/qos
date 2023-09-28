#![feature(prelude_2024)]
#![no_std]

use limine::{SMPInfo, KernelAddress, MemoryMap};

pub mod limine;

pub enum BootInfo {
    Limine(LimineBootInfo),
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimineBootInfo {
    pub smp_info: Option<SMPInfo>,
    pub kernel_address: Option<KernelAddress>,
    pub memory_map: Option<MemoryMap>,
    //pub efi_info: Option<limine::EFIInfo>,
}

impl BootInfo {
    pub fn new_from_limine(boot_info: LimineBootInfo) -> Self {
        BootInfo::Limine(boot_info)
    }
}