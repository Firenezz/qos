#![allow(dead_code)]

const LIMINE_REVISION: u64 = 0x0000_0005;

pub(crate) static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(LIMINE_REVISION);

/// SMP Request:
/// 
/// This is a request for the SMP information.
/// It constains the information of the bootstrapped processor.
/// 
/// Use cpus() to get a list of all processors including the bootstrapped processor.
pub(crate) static SMP_REQUEST: limine::SmpRequest = limine::SmpRequest::new(LIMINE_REVISION);

/// KERNEL_ADDRESS Request:
/// 
/// This is a request for the kernel address.
/// It contains the physical and virtual address of the kernel.
pub(crate) static KERNEL_ADDRESS_REQUEST: limine::KernelAddressRequest = limine::KernelAddressRequest::new(LIMINE_REVISION);

/// EFI_SYSTEM_TABLE Request:
/// 
/// This is a request for the EFI_SYSTEM_TABLE address.
/// 'address' - Address of EFI system table
pub(crate) static EFI_SYSTEM_TABLE_REQUEST: limine::EfiSystemTableRequest = limine::EfiSystemTableRequest::new(LIMINE_REVISION);

/// MEMMAP Request:
/// 
/// This is a request for the memory map.
/// 
pub(crate) static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(LIMINE_REVISION);

/// DEVICE_TREE_BLOB Request:
/// 
/// This is a request for the device tree blob.
/// 
/// Note: If the DTB cannot be found, the response will not be generated.
///
/// Note: Information contained in the /chosen node may not reflect the information given by bootloader tags, and as such the /chosen node properties should be ignored.
pub(crate) static DEVICE_TREE_BLOB_REQUEST: limine::DtbRequest = limine::DtbRequest::new(LIMINE_REVISION);