//! Environment-call helpers.

/// Trigger a system call (`ECALL`) with no explicit register setup.
///
/// # Safety
///
/// Caller must follow the ABI expected by the handler.
#[inline(always)]
pub unsafe fn ecall() {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    // SAFETY: caller upholds the ecall ABI / trap-handler contract.
    unsafe {
        core::arch::asm!("ecall", options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    unimplemented!("ecall is only available on RISC-V");
}
