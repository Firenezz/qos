use common_boot::limine::MemoryMapEntry;
use limine::MemmapEntry;



pub(crate) mod limine_request {

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

    /// RSDP Request:
    /// 
    /// This is a request for the RSDP.
    /// 
    /// Note: If the RSDP cannot be found, the response will not be generated.
    pub(crate) static RSDP_REQUEST: limine::RsdpRequest = limine::RsdpRequest::new(LIMINE_REVISION);
}

pub struct LimineBootInfo {
    pub rsdp_response: Option<&'static limine::RsdpResponse>,
    pub dtb_response: Option<&'static limine::DtbResponse>,
    pub memmap_response: Option<&'static limine::MemmapResponse>,
    pub smp_response: Option<&'static limine::SmpResponse>,
    pub kernel_address_response: Option<&'static limine::KernelAddressResponse>,
    pub framebuffer_response: Option<&'static limine::FramebufferResponse>,
    pub efi_system_table_response: Option<&'static limine::EfiSystemTableResponse>,
}

fn gather_limine_boot_info() -> LimineBootInfo {

    let limine_boot_info = LimineBootInfo {
        rsdp_response: limine_request::RSDP_REQUEST.get_response().get(),
        dtb_response: limine_request::DEVICE_TREE_BLOB_REQUEST.get_response().get(),
        memmap_response: limine_request::MEMMAP_REQUEST.get_response().get(),
        smp_response: limine_request::SMP_REQUEST.get_response().get(),
        kernel_address_response: limine_request::KERNEL_ADDRESS_REQUEST.get_response().get(),
        framebuffer_response: limine_request::FRAMEBUFFER_REQUEST.get_response().get(),
        efi_system_table_response: limine_request::EFI_SYSTEM_TABLE_REQUEST.get_response().get(),
    };

    limine_boot_info
}

pub(crate) fn create_limine_common_boot_info() -> common_boot::LimineBootInfo {

    let limine_info = gather_limine_boot_info();

    from_limin_boot_info(limine_info)
}

fn from_limin_boot_info(limine_boot_info: LimineBootInfo) -> common_boot::LimineBootInfo {
    use common_boot::limine::MemoryType;

    

    fn get_smp_info(smp_response: &limine::SmpResponse) -> common_boot::limine::SMPInfo {
        
        let sliced_cpus = unsafe { core::slice::from_raw_parts_mut(smp_response.cpus.as_ptr(), smp_response.cpu_count as usize) }; // this should be always present
        let sliced_cpus = convert(sliced_cpus);

        let smp_info = common_boot::limine::SMPInfo {
            hart_id: smp_response.bsp_hartid,
            cpus: sliced_cpus,
        };

        smp_info
    }

    fn get_memory_map(memmap_response: &limine::MemmapResponse) -> common_boot::limine::MemoryMap {
        let entries = unsafe { core::slice::from_raw_parts_mut(memmap_response.entries.as_ptr(), memmap_response.entry_count as usize) };
        let entries = convert::<MemmapEntry, MemoryMapEntry>(entries);

        let memory_map = common_boot::limine::MemoryMap {
            entries,
        };

        memory_map
    }

    let smp_info = match limine_boot_info.smp_response {
        Some(smp_response) => Some(get_smp_info(smp_response)),
        None => None
    };

    let kernel_address = match limine_boot_info.kernel_address_response {
        Some(kernel_address_response) => Some(common_boot::limine::KernelAddress {
            physical_address: kernel_address_response.physical_base,
            virtual_address: kernel_address_response.virtual_base,
        }),
        None => None
    };

    let memory_map = match limine_boot_info.memmap_response {
        Some(memmap_response) => Some(get_memory_map(memmap_response)),
        None => None
    };

    common_boot::LimineBootInfo {
        smp_info,
        kernel_address,
        memory_map,
    }
}

fn convert<TSrcData, TDestData>(src_data_ptrs: &mut [limine::NonNullPtr<TSrcData>]) -> &mut [&mut TDestData] {
    use core::slice;
    // Create a mutable slice of raw pointers to TSrcData
    let src_raw_ptrs: &mut [*mut TSrcData] = unsafe {
        slice::from_raw_parts_mut(src_data_ptrs.as_mut_ptr() as *mut *mut TSrcData, src_data_ptrs.len())
    };

    // Create a mutable slice of raw pointers to TDestData
    let dest_raw_ptrs: &mut [*mut TDestData] = unsafe {
        slice::from_raw_parts_mut(src_raw_ptrs as *mut [*mut TSrcData] as *mut *mut TDestData, src_data_ptrs.len())
    };

    // Convert the mutable slice of raw pointers to TDestData into a mutable slice of references
    let dest_data: &mut [&mut TDestData] = unsafe {
        slice::from_raw_parts_mut(dest_raw_ptrs as *mut [*mut TDestData] as *mut &mut TDestData, src_data_ptrs.len())
    };

    dest_data
}