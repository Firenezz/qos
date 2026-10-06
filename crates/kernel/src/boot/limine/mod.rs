//! Limine protocol adapter: requests + handoff → [`crate::boot::BootInfo`].

use core::mem::MaybeUninit;
use core::num::NonZeroU64;

use limine::{
    mp::MpInfo,
    request::{
        DtbResponse, EfiResponse, ExecutableAddressResponse, ExecutableCmdlineResponse,
        FramebufferResponse, HhdmRepsonse, MemmapResponse, MpResponse, RsdpResponse,
    },
};

use crate::boot::{Architecture, BootInfo, BootLoader, Capability, CpuInfo};
#[cfg(target_arch = "riscv64")]
use crate::boot::{Capability::Absent, info::KernelAddressInfo};
use crate::startup;

use self::memmap::LimineMemoryKind;

mod entry;
mod memmap;

pub use entry::ENTRY_POINT_REQUEST;

static mut CPU_MAP: Option<&[&MpInfo]> = None;

/// Boot-time `CpuInfo` snapshot (`CpuMap` needs a long borrow; heap is not ready yet).
///
/// Lives in `.bss.reclaim` → linker `.boot_reclaim` (NOLOAD). Free
/// `[__boot_reclaim_start, __boot_reclaim_end)` only after [`BootInfo`] is
/// consumed and nothing still points at this array.
const MAX_BOOT_CPUS: usize = 256;
#[unsafe(link_section = ".bss.reclaim")]
static mut BOOT_CPU_INFO: [MaybeUninit<CpuInfo>; MAX_BOOT_CPUS] =
    [const { MaybeUninit::uninit() }; MAX_BOOT_CPUS];

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
    pub(crate) static EXECUTABLE_CMDLINE_REQUEST: ExecutableCmdlineRequest =
        ExecutableCmdlineRequest::new();

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
    pub executable_cmdline_response: Option<&'static ExecutableCmdlineResponse>,
}

#[inline(always)]
fn cast_pointer<T: alloc::fmt::Debug>(pointer: u64) -> Option<&'static T> {
    let test = NonZeroU64::new(pointer).map(|pointer| unsafe { &*(pointer.get() as *const T) });
    log::debug!("Cast pointer: {:#x} -> {:?}", pointer, test);
    test
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
            cmd_line: limine_boot_info
                .executable_cmdline_response
                .map_or(Capability::Absent, |cmd_line| {
                    Capability::Present(cmd_line.cmdline())
                }),
            rsdp: limine_boot_info
                .rsdp_response
                .map_or(Capability::Absent, |rsdp| {
                    cast_pointer(rsdp.address as u64)
                        .map_or(Capability::Absent, |rsdp| Capability::Present(rsdp))
                }),
            kernel_address: limine_boot_info.kernel_address_response.map_or(
                None,
                |kernel_address| {
                    Some(KernelAddressInfo {
                        physical_base: cast_pointer(kernel_address.physical_base as u64)
                            .map_or(None, |kernel_address| Some(kernel_address)),
                        virtual_base: cast_pointer(kernel_address.virtual_base as u64)
                            .map_or(None, |kernel_address| Some(kernel_address)),
                    })
                },
            ),
            cpus: limine_boot_info.smp_response.map_or(Absent, map_smp_cpus),
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
            cmd_line: limine_boot_info
                .executable_cmdline_response
                .map_or(Capability::Absent, |cmd_line| {
                    Capability::Present(cmd_line.cmdline())
                }),
            rsdp: limine_boot_info
                .rsdp_response
                .map_or(Capability::Absent, |rsdp| {
                    cast_pointer(rsdp.address as u64)
                        .map_or(Capability::Absent, |rsdp| Capability::Present(rsdp))
                }),
            kernel_address: limine_boot_info.kernel_address_response.map_or(
                None,
                |kernel_address| {
                    Some(KernelAddressInfo {
                        physical_base: cast_pointer(kernel_address.physical_base as u64)
                            .map_or(None, |kernel_address| Some(kernel_address)),
                        virtual_base: cast_pointer(kernel_address.virtual_base as u64)
                            .map_or(None, |kernel_address| Some(kernel_address)),
                    })
                },
            ),
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
    limine_boot_info
        .efi_system_table_response
        .map_or(Capability::Absent, |efi| {
            cast_pointer(efi.address as u64)
                .map_or(Capability::Absent, |efi| Capability::Present(efi))
        })
}

#[cfg(target_arch = "riscv64")]
fn map_smp_cpus(smp: &'static MpResponse) -> Capability<crate::boot::CpuMap> {
    use crate::boot::CpuMap;

    let bsp_hartid = smp.bsp_hartid;
    let limine_cpus = smp.cpus();
    let n = limine_cpus.len().min(MAX_BOOT_CPUS);
    if limine_cpus.len() > MAX_BOOT_CPUS {
        log::warn!(
            "Limine reported {} CPUs; only first {MAX_BOOT_CPUS} kept in BootInfo",
            limine_cpus.len()
        );
    }

    // SAFETY: handoff is single-threaded on the BSP; we `write` `n` elements
    // before forming the slice (valid until `.boot_reclaim` is freed).
    unsafe {
        let slots = core::ptr::addr_of_mut!(BOOT_CPU_INFO);
        for (i, cpu) in limine_cpus.iter().take(n).enumerate() {
            (*slots)[i].write(CpuInfo::new(
                cpu.processor_id as u32,
                cpu.hartid as u64,
                Capability::Absent,
                cpu.hartid == bsp_hartid,
            ));
        }
        Capability::Present(CpuMap {
            cpus: core::slice::from_raw_parts(slots.cast::<CpuInfo>(), n),
        })
    }
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
        executable_cmdline_response: EXECUTABLE_CMDLINE_REQUEST.response().take(),
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
    let actual_revision = BASE_REVISION
        .actual_revision()
        .expect("Base revision not supported");
    if !BASE_REVISION.is_supported() || actual_revision < 4 {
        crate::halt();
    }

    let limine_info = gather_limine_boot_info();

    let hhdm_offset = limine_info
        .hhdm_response
        .map_or(None, |hhdm| NonZeroU64::new(hhdm.offset));

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
