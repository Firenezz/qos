//! Bootloader-agnostic boot information.
//!
//! Limine (or another boot protocol) converts into this via [`From`] / [`TryFrom`].
//! Optional firmware services use [`Capability`].

use crate::boot::capabilities::Capability;
use crate::boot::memory::BootMemorySnapshot;

/// Snapshot of what the bootloader handed the kernel.
///
/// Pointers/slices that come from bootloader memory are only valid until that
/// memory is reclaimed. Owned copies (e.g. [`BootMemorySnapshot`]) remain valid.
#[derive(Debug, Clone)]
pub struct BootInfo {
    pub architecture: Architecture,

    /// Hardware id of the bootstrap CPU (hart id, APIC id, …).
    pub bsp_hw_id: u64,

    /// Physical base of the kernel image, if known.
    pub kernel_address: Option<KernelAddressInfo>,

    /// Higher-half direct map offset (phys + offset → virt), if provided.
    pub hhdm_offset: Option<u64>,

    /// Owned bootloader memmap; installed by [`crate::kernel::kernel_main`].
    pub memory: BootMemorySnapshot,

    pub cpus: Capability<CpuMap>,

    pub framebuffer: Capability<FramebufferInfo>,
    /// Physical address of the ACPI RSDP, if any. The absence of this capability doesn't indicate the absence of ACPI,
    ///
    /// only that the bootloader didn't provide it. It could be in uefi system table or in a separate table.
    pub rsdp: Capability<&'static ()>,
    /// Virtual pointer to an FDT blob, if any.
    pub dtb: Capability<*const u8>,
    /// Virtual/physical pointer to the EFI system table, if any.
    pub efi_system_table: Capability<*const ()>,
    /// Command line string, if any.
    pub cmd_line: Capability<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelAddressInfo {
    pub physical_base: Option<*const ()>,
    pub virtual_base: Option<*const ()>,
}

impl BootInfo {
    pub const fn empty() -> Self {
        Self {
            architecture: Architecture::Unknown,
            bsp_hw_id: 0,
            kernel_address: None,
            hhdm_offset: None,
            memory: BootMemorySnapshot::empty(),
            cpus: Capability::Absent,
            framebuffer: Capability::Absent,
            rsdp: Capability::Absent,
            dtb: Capability::Absent,
            efi_system_table: Capability::Absent,
            cmd_line: Capability::Absent,
        }
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    RiscV64,
    Other(&'static str),
    Unknown,
}

impl Architecture {
    pub const fn native() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            Self::Aarch64
        }
        #[cfg(target_arch = "riscv64")]
        {
            Self::RiscV64
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        {
            Self::Unknown
        }
    }
}

/// CPU list as reported by the bootloader.
#[derive(Debug, Clone, Copy)]
pub struct CpuMap {
    pub cpus: &'static [CpuInfo],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuInfo {
    /// Dense logical index assigned by the kernel later (0 = BSP once assigned).
    pub cpu_id: u32,
    /// Architecture hardware id (hart id, LAPIC id, MPIDR, …).
    pub hw_id: u64,
    /// ACPI Processor UID when present.
    pub acpi_uid: Capability<u32>,
    pub is_bsp: bool,
}

impl CpuInfo {
    pub const fn new(cpu_id: u32, hw_id: u64, acpi_uid: Capability<u32>, is_bsp: bool) -> Self {
        Self {
            cpu_id,
            hw_id,
            acpi_uid,
            is_bsp,
        }
    }
}

impl Into<crate::kernel::cpu::ProcessorInfo> for CpuInfo {
    fn into(self) -> crate::kernel::cpu::ProcessorInfo {
        crate::kernel::cpu::ProcessorInfo::new(
            self.cpu_id as usize,
            self.hw_id as usize,
            match self.acpi_uid {
                Capability::Present(uid) => Some(uid as u64),
                Capability::Absent => None,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
}
