//! Static BSP trap stack — range lives in [`HartLocal`] (`tp`), swap uses `sscratch`.

use riscv_rs::csr::sscratch;

use crate::arch::riscv::device::cpu::{self, HartLocal};

/// Bytes reserved for the supervisor trap stack (grows down).
pub const TRAP_STACK_SIZE: usize = 16 * 1024;

#[repr(C, align(16))]
struct TrapStackStorage {
    bytes: [u8; TRAP_STACK_SIZE],
}

/// BSS trap stack for the bootstrap hart.
#[unsafe(link_section = ".bss.trap_stack")]
static mut TRAP_STACK: TrapStackStorage = TrapStackStorage {
    bytes: [0; TRAP_STACK_SIZE],
};

#[inline]
fn stack_base() -> usize {
    // SAFETY: address-of a static; not dereferencing the contents here.
    unsafe { core::ptr::addr_of_mut!(TRAP_STACK.bytes) as usize }
}

#[inline]
fn stack_top() -> usize {
    stack_base() + TRAP_STACK_SIZE
}

/// Install the BSP trap stack into [`HartLocal`], set `tp`, and arm `sscratch`.
///
/// Stack top is stored in `HartLocal` (via `tp`); `sscratch` mirrors the top for
/// the entry `csrrw` swap.
///
/// # Safety
///
/// Call once on the BSP before relying on traps.
pub unsafe fn install() {
    let base = stack_base();
    let top = stack_top();

    // SAFETY: BSP only; exclusive access during bring-up.
    let hart = unsafe { &mut *core::ptr::addr_of_mut!(cpu::BSP_HART) };
    hart.trap_stack_base = base;
    hart.trap_stack_top = top;
    hart.trap_depth = 0;

    // SAFETY: `hart` is the BSP `HartLocal` for the life of the kernel.
    unsafe {
        cpu::set_tp(hart as *mut HartLocal);
    }
    sscratch::write_raw(top as u64);
}
