//! Limine protocol adapter: requests + handoff → [`crate::boot::BootInfo`].

use limine::request::{
    DtbResponse, EfiResponse, ExecutableAddressResponse, FramebufferResponse, MemmapResponse,
    MpResponse, RsdpResponse,
};

use crate::boot::{Architecture, BootInfo, Capability};

mod entry;

pub use entry::ENTRY_POINT_REQUEST;

pub(crate) mod limine_request {
    use limine::request::*;

    #[unsafe(link_section = ".requests")]
    pub(crate) static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static SMP_REQUEST: MpRequest = MpRequest::new(0);

    #[unsafe(link_section = ".requests")]
    pub(crate) static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest =
        ExecutableAddressRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static EFI_SYSTEM_TABLE_REQUEST: EfiRequest = EfiRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static DEVICE_TREE_BLOB_REQUEST: DtbRequest = DtbRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();
}

pub struct LimineBootInfo {
    pub rsdp_response: Option<&'static RsdpResponse>,
    pub dtb_response: Option<&'static DtbResponse>,
    pub memmap_response: Option<&'static MemmapResponse>,
    pub smp_response: Option<&'static MpResponse>,
    pub kernel_address_response: Option<&'static ExecutableAddressResponse>,
    pub framebuffer_response: Option<&'static FramebufferResponse>,
    pub efi_system_table_response: Option<&'static EfiResponse>,
}

impl TryFrom<LimineBootInfo> for BootInfo {
    type Error = ();

    fn try_from(limine_boot_info: LimineBootInfo) -> Result<BootInfo, Self::Error> {
        Ok(Self {
            architecture: Architecture::RiscV64,
            bsp_hw_id: limine_boot_info
                .smp_response
                .expect("Bootstrapped processor required for booting")
                .bsp_hartid as u64,
            efi_system_table: map_efi_system_table(&limine_boot_info),
            ..Default::default()
        })
    }
}

#[inline(always)]
fn map_efi_system_table(limine_boot_info: &LimineBootInfo) -> Capability<*const ()> {
    limine_boot_info.efi_system_table_response.map_or(
        Capability::Absent,
        |efi| Capability::Present(efi.address),
    )
}

fn gather_limine_boot_info() -> LimineBootInfo {
    LimineBootInfo {
        rsdp_response: limine_request::RSDP_REQUEST.response().take(),
        dtb_response: limine_request::DEVICE_TREE_BLOB_REQUEST.response().take(),
        memmap_response: limine_request::MEMMAP_REQUEST.response().take(),
        smp_response: limine_request::SMP_REQUEST.response().take(),
        kernel_address_response: limine_request::KERNEL_ADDRESS_REQUEST.response().take(),
        framebuffer_response: limine_request::FRAMEBUFFER_REQUEST.response().take(),
        efi_system_table_response: limine_request::EFI_SYSTEM_TABLE_REQUEST.response().take(),
    }
}

/// Collect Limine responses and enter [`crate::boot::kernel_main`].
pub fn handoff() -> ! {
    let limine_info = gather_limine_boot_info();
    match BootInfo::try_from(limine_info) {
        Ok(info) => crate::boot::kernel_main(info),
        Err(()) => crate::halt(),
    }
}
