//! Per-hart state. Armed into `sscratch` after boot.
//!
//! Trap entry swaps `sscratch` ↔ `tp` so the handler sees percpu in `tp` and
//! parks TLS in `sscratch`.

use riscv_rs::csr::sscratch;

/// Offsets into [`HartLocal`] for trap asm (`tp`-relative while in the handler).
pub mod off {
    pub const SCRATCH: usize = 0;
    pub const TRAP_STACK_TOP: usize = 8;
    pub const TRAP_STACK_BASE: usize = 16;
    pub const TRAP_DEPTH: usize = 24;
    pub const HART_ID: usize = 32;
}

/// Hart-local kernel data. Written into `sscratch` when the hart is fully booted.
#[repr(C)]
pub struct HartLocal {
    /// Entry scratch (temp regs / interrupted SP across U-mode stack switch).
    pub scratch: usize,
    /// Top of the trap stack (16-byte aligned); loaded as `sp` on U→S.
    pub trap_stack_top: usize,
    /// Base of the trap stack buffer.
    pub trap_stack_base: usize,
    /// Nest depth (reserved).
    pub trap_depth: u8,
    pub _pad: [u8; 7],
    /// Hardware hart id (Limine / DT / ACPI).
    pub hart_id: u64,
}

const _: () = assert!(core::mem::offset_of!(HartLocal, scratch) == off::SCRATCH);
const _: () = assert!(core::mem::offset_of!(HartLocal, trap_stack_top) == off::TRAP_STACK_TOP);
const _: () = assert!(core::mem::offset_of!(HartLocal, trap_stack_base) == off::TRAP_STACK_BASE);
const _: () = assert!(core::mem::offset_of!(HartLocal, trap_depth) == off::TRAP_DEPTH);
const _: () = assert!(core::mem::offset_of!(HartLocal, hart_id) == off::HART_ID);

impl HartLocal {
    pub const fn empty() -> Self {
        Self {
            scratch: 0,
            trap_stack_top: 0,
            trap_stack_base: 0,
            trap_depth: 0,
            _pad: [0; 7],
            hart_id: 0,
        }
    }
}

/// Bootstrap hart's `HartLocal` (APs get their own later).
pub static mut BSP_HART: HartLocal = HartLocal::empty();

/// Current hart's [`HartLocal`].
///
/// Uses `sscratch` when armed; otherwise the BSP slot (bring-up / early traps).
///
/// # Safety
///
/// If `sscratch` is non-zero it must point at a live [`HartLocal`].
#[inline(always)]
pub unsafe fn current() -> &'static mut HartLocal {
    let ptr = sscratch::read_raw() as *mut HartLocal;
    if !ptr.is_null() {
        // SAFETY: caller guarantees an armed `sscratch` is a valid hart pointer.
        unsafe { &mut *ptr }
    } else {
        // SAFETY: BSP bring-up; single-threaded w.r.t. this static.
        unsafe { &mut *core::ptr::addr_of_mut!(BSP_HART) }
    }
}

/// Arm `sscratch` with `hart` so trap entry can find per-hart state.
///
/// Leave `sscratch` as 0 until the hart is ready for the non-early trap path.
///
/// # Safety
///
/// `hart` must remain valid for the life of the hart.
#[inline(always)]
pub unsafe fn arm_sscratch(hart: *mut HartLocal) {
    sscratch::write_raw(hart as u64);
}
