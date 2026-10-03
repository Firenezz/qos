//! Limine protocol adapter: requests + handoff → [`crate::boot::BootInfo`].

use core::num::NonZeroU64;

use limine::{mp::MpInfo, request::{
    DtbResponse, EfiResponse, ExecutableAddressResponse, FramebufferResponse, MemmapResponse,
    MpResponse, RsdpResponse, HhdmRepsonse,
}};

use crate::boot::{Architecture, BootInfo, BootLoader, Capability};
use crate::startup;

use self::memmap::LimineMemoryKind;

mod entry;
mod memmap;

pub use entry::ENTRY_POINT_REQUEST;

static mut CPU_MAP: Option<&[&MpInfo]> = None;

use limine_request::*;


pub(crate) mod limine_request {
    use limine::request::*;

    #[unsafe(link_section = ".requests")]
    pub(crate) static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

    #[unsafe(link_section = ".requests")]
    pub(crate) static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::with_revision(4);

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

    #[unsafe(link_section = ".requests")]
    pub(crate) static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
}

pub struct LimineBootInfo {
    pub rsdp_response: Option<&'static RsdpResponse>,
    pub dtb_response: Option<&'static DtbResponse>,
    pub memmap_response: Option<&'static MemmapResponse>,
    pub smp_response: Option<&'static MpResponse>,
    pub kernel_address_response: Option<&'static ExecutableAddressResponse>,
    pub framebuffer_response: Option<&'static FramebufferResponse>,
    pub efi_system_table_response: Option<&'static EfiResponse>,
    pub hhdm_response: Option<&'static HhdmRepsonse>,
}

impl TryFrom<LimineBootInfo> for BootInfo {
    type Error = ();

    fn try_from(limine_boot_info: LimineBootInfo) -> Result<BootInfo, Self::Error> {

        if limine_boot_info.memmap_response.is_some() {
            log::trace!("Initializing memory map from Limine");
            startup::memory::init_from_boot(&limine_boot_info);
        }

        #[cfg(target_arch = "riscv64")]
        return Ok(Self {
            architecture: Architecture::RiscV64,
            bsp_hw_id: limine_boot_info
                .smp_response
                .expect("Bootstrapped processor required for booting")
                .bsp_hartid as u64,
            efi_system_table: map_efi_system_table(&limine_boot_info),
            hhdm_offset: limine_boot_info.hhdm_response.map(|hhdm| hhdm.offset),
            ..Default::default()
        });
        #[cfg(target_arch = "x86_64")]
        return Ok(Self {
            architecture: Architecture::X86_64,
            bsp_hw_id: limine_boot_info
                .smp_response
                .expect("Bootstrapped processor required for booting")
                .bsp_lapic_id as u64,
            efi_system_table: map_efi_system_table(&limine_boot_info),
            hhdm_offset: limine_boot_info.hhdm_response.map(|hhdm| hhdm.offset),
            ..Default::default()
        });
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64",
            target_arch = "loongarch64"
        )))]
        return Err(());
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
        rsdp_response: RSDP_REQUEST.response().take(),
        dtb_response: DEVICE_TREE_BLOB_REQUEST.response().take(),
        memmap_response: MEMMAP_REQUEST.response().take(),
        smp_response: SMP_REQUEST.response().take(),
        kernel_address_response: KERNEL_ADDRESS_REQUEST.response().take(),
        framebuffer_response: FRAMEBUFFER_REQUEST.response().take(),
        efi_system_table_response: EFI_SYSTEM_TABLE_REQUEST.response().take(),
        hhdm_response: HHDM_REQUEST.response().take(),
    }
}

fn memmap_entry_kind(type_: u64) -> &'static str {
    LimineMemoryKind(type_).debug_name()
}

/// Log the Limine memory map for bring-up / debugging.
fn debug_bootloader_memmap(limine_info: &LimineBootInfo) {
    let Some(memmap) = limine_info.memmap_response else {
        log::warn!("Limine memory map: no response");
        return;
    };

    let entries = memmap.entries();
    log::info!("Limine memory map ({} entries):", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        let end = entry.base.saturating_add(entry.length.saturating_sub(1));
        let kind = memmap_entry_kind(entry.type_);
        if kind == "unknown" {
            log::info!(
                "  [{i}] {:#018x}..={:#018x} ({:#x} bytes) type={} ({})",
                entry.base,
                end,
                entry.length,
                entry.type_,
                kind,
            );
        } else {
            log::info!(
                "  [{i}] {:#018x}..={:#018x} ({:#x} bytes) {}",
                entry.base,
                end,
                entry.length,
                kind,
            );
        }
    }
}

/// Collect Limine responses and enter [`crate::boot::kernel_main`].
pub fn handoff() -> ! {
    let actual_revision = BASE_REVISION.actual_revision().expect("Base revision not supported");
    if !BASE_REVISION.is_supported() || actual_revision < 4 {
        crate::halt();
    }

    let limine_info = gather_limine_boot_info();

    let hhdm_offset = limine_info.hhdm_response.map_or(None, |hhdm| NonZeroU64::new(hhdm.offset));

    // call pre-bootloader initialization for early logging and other setup
    crate::kernel::pre_bootloader_init(hhdm_offset);
    if let Some(smp) = limine_info.smp_response {
        unsafe {
            CPU_MAP = Some(smp.cpus());
        }
    }

    debug_bootloader_memmap(&limine_info);

    match BootInfo::try_from(limine_info) {
        Ok(info) => crate::kernel::kernel_main(info),
        Err(()) => crate::halt(),
    }
}

pub struct LimineBootloader;

impl BootLoader for LimineBootloader {
    fn handoff(&self) -> ! {
        todo!()
    }

    fn bootstrap(&self) -> Result<(), &'static str> {
        match unsafe { CPU_MAP } {
            Some(cpu_map) => {
                for cpu in cpu_map {
                    // SAFETY: safe to also bootstrap BSP with this function since it is already running here
                    // (Limine parks the harts in a loop that checks the function pointer and if it is not null, it will jump to it)
                    cpu.bootstrap(entry::limine_ap_trampoline, 0);
                }
            }
            None => return Err("No CPU map found"),
        }
        Ok(())
    }
}