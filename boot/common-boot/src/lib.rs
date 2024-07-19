#![no_std]

use limine::{KernelAddress, MemoryMap, SMPInfo};

pub mod limine;
//pub mod common;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootLoader {
    Limine,
    Unknown,
}

#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BootInfo {
    pub loader: Option<BootLoader>,
    pub smp_info: Option<SMPInfo>,
    pub kernel_address: Option<KernelAddress>,
    pub memory_map: Option<MemoryMap>,
    //pub efi_info: Option<limine::EFIInfo>,
}
