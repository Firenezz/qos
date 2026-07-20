//! Per-hart state pointed to by `tp` while in the kernel.

/// Offsets into [`HartLocal`] for trap asm (`tp`-relative).
pub mod off {
    pub const TRAP_STACK_TOP: usize = 0;
    pub const TRAP_STACK_BASE: usize = 8;
    pub const TRAP_DEPTH: usize = 16;
    pub const HART_ID: usize = 24;
}

/// Hart-local kernel data. `tp` points here after [`crate::arch::riscv::interrupt::init`].
#[repr(C)]
pub struct HartLocal {
    /// Top of the trap stack (16-byte aligned); also kept in `sscratch` for swaps.
    pub trap_stack_top: usize,
    /// Base of the trap stack buffer.
    pub trap_stack_base: usize,
    /// Nest depth (reserved; entry uses address-range checks for now).
    pub trap_depth: u8,
    pub _pad: [u8; 7],
    /// Hardware hart id (Limine / DT / ACPI).
    pub hart_id: u64,
}

const _: () = assert!(core::mem::offset_of!(HartLocal, trap_stack_top) == off::TRAP_STACK_TOP);
const _: () = assert!(core::mem::offset_of!(HartLocal, trap_stack_base) == off::TRAP_STACK_BASE);
const _: () = assert!(core::mem::offset_of!(HartLocal, trap_depth) == off::TRAP_DEPTH);
const _: () = assert!(core::mem::offset_of!(HartLocal, hart_id) == off::HART_ID);

impl HartLocal {
    pub const fn empty() -> Self {
        Self {
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

/// Current hart's [`HartLocal`] via `tp`.
///
/// # Safety
///
/// `tp` must point at a valid [`HartLocal`] (set by interrupt/`stack` install).
#[inline(always)]
pub unsafe fn current() -> &'static mut HartLocal {
    let tp: *mut HartLocal;
    // SAFETY: caller guarantees `tp` is armed.
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp, options(nomem, nostack, preserves_flags));
        &mut *tp
    }
}

/// Load `tp` with `hart`.
#[inline(always)]
pub unsafe fn set_tp(hart: *mut HartLocal) {
    unsafe {
        core::arch::asm!("mv tp, {}", in(reg) hart, options(nomem, nostack, preserves_flags));
    }
}
