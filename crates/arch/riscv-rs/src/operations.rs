//! Privilege and barrier instruction wrappers (`EBREAK`, `ECALL`, `FENCE`, …).

/// Expand a nullary instruction into a thin `asm!` wrapper.
///
/// ```ignore
/// instruction! {
///     /// docs
///     unsafe fn ebreak = "ebreak", options(nomem, nostack, preserves_flags)
/// }
///
/// instruction! {
///     /// docs
///     fn fence = "fence", options(nostack)
/// }
/// ```
macro_rules! instruction {
    (
        $(#[$attr:meta])*
        unsafe fn $name:ident = $asm:literal $(, $($options:tt)*)?
    ) => {
        $(#[$attr])*
        #[inline(always)]
        pub unsafe fn $name() {
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            // SAFETY: caller upholds this instruction's documented contract.
            unsafe {
                core::arch::asm!($asm $(, $($options)*)?);
            }
            #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
            {
                let _ = $asm;
                unimplemented!(concat!(stringify!($name), " is only available on RISC-V"));
            }
        }
    };
    (
        $(#[$attr:meta])*
        fn $name:ident = $asm:literal $(, $($options:tt)*)?
    ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name() {
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            // SAFETY: instruction has no operands and is always valid to execute.
            unsafe {
                core::arch::asm!($asm $(, $($options)*)?);
            }
            #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
            {
                let _ = $asm;
                unimplemented!(concat!(stringify!($name), " is only available on RISC-V"));
            }
        }
    };
}

instruction! {
    /// Generate a breakpoint exception (`EBREAK`).
    ///
    /// Transfers control to a debugger or the breakpoint trap handler.
    ///
    /// # Safety
    ///
    /// The surrounding environment must be prepared to handle a breakpoint trap
    /// (or the hart will take an unhandled exception).
    unsafe fn ebreak = "ebreak", options(nomem, nostack, preserves_flags)
}

instruction! {
    /// Environment call (`ECALL`).
    ///
    /// Raises an environment-call exception for the current privilege mode
    /// (U-mode → `ecall` from U, S-mode → from S, M-mode → from M).
    ///
    /// # Safety
    ///
    /// The caller must follow the ABI expected by the handler (argument registers,
    /// and that a trap handler is installed).
    unsafe fn ecall = "ecall", options(nomem, nostack, preserves_flags)
}

instruction! {
    /// Full memory fence (`FENCE`).
    ///
    /// Orders all prior memory accesses before subsequent ones on this hart.
    fn fence = "fence", options(nostack)
}

instruction! {
    /// Instruction fence (`FENCE.I`).
    ///
    /// Synchronizes this hart's instruction fetch with prior stores (I-cache
    /// coherency after writing executable code).
    fn fence_i = "fence.i", options(nostack)
}

instruction! {
    /// Flush the instruction cache pipeline view.
    ///
    /// Alias of [`fence_i`] — RISC-V uses `FENCE.I` for this; there is no
    /// separate `flush.i` opcode in the base ISA.
    fn flush_icache = "fence.i", options(nostack)
}

instruction! {
    /// Wait for interrupt (`WFI`).
    ///
    /// Hints that this hart may enter a low-power state until an interrupt
    /// becomes pending. May return spuriously.
    fn wfi = "wfi", options(nomem, nostack, preserves_flags)
}

instruction! {
    /// No operation (`NOP`).
    fn nop = "nop", options(nomem, nostack, preserves_flags)
}

instruction! {
    /// Local TLB fence for all VAs and ASIDs (`SFENCE.VMA` with `rs1 = x0`, `rs2 = x0`).
    ///
    /// Orders prior page-table stores and invalidates cached address translations
    /// on **this hart only**. Other harts need an IPI (or SBI remote fence) plus
    /// their own `sfence.vma`.
    ///
    /// # Safety
    ///
    /// Caller must execute this in a mode allowed to manage address translation
    /// (typically S-mode or M-mode) after the relevant page-table updates.
    unsafe fn sfence_vma = "sfence.vma", options(nostack)
}

instruction! {
    /// Full local TLB flush — alias of [`sfence_vma`].
    ///
    /// # Safety
    ///
    /// Same contract as [`sfence_vma`].
    unsafe fn flush_tlb = "sfence.vma", options(nostack)
}

/// Local TLB fence for one virtual address (`SFENCE.VMA rs1, x0`).
///
/// # Safety
///
/// Same privilege / ordering contract as [`sfence_vma`].
#[inline(always)]
pub unsafe fn sfence_vma_vaddr(vaddr: usize) {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    // SAFETY: caller upholds the `sfence.vma` contract; `vaddr` is only an operand.
    unsafe {
        core::arch::asm!("sfence.vma {}, zero", in(reg) vaddr, options(nostack));
    }
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    {
        let _ = vaddr;
        unimplemented!("sfence_vma_vaddr is only available on RISC-V");
    }
}

/// Local TLB fence for one ASID (`SFENCE.VMA x0, rs2`).
///
/// # Safety
///
/// Same privilege / ordering contract as [`sfence_vma`].
#[inline(always)]
pub unsafe fn sfence_vma_asid(asid: usize) {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    // SAFETY: caller upholds the `sfence.vma` contract; `asid` is only an operand.
    unsafe {
        core::arch::asm!("sfence.vma zero, {}", in(reg) asid, options(nostack));
    }
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    {
        let _ = asid;
        unimplemented!("sfence_vma_asid is only available on RISC-V");
    }
}

/// Local TLB fence for one VA under one ASID (`SFENCE.VMA rs1, rs2`).
///
/// # Safety
///
/// Same privilege / ordering contract as [`sfence_vma`].
#[inline(always)]
pub unsafe fn sfence_vma_vaddr_asid(vaddr: usize, asid: usize) {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    // SAFETY: caller upholds the `sfence.vma` contract; args are only operands.
    unsafe {
        core::arch::asm!(
            "sfence.vma {}, {}",
            in(reg) vaddr,
            in(reg) asid,
            options(nostack)
        );
    }
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    {
        let _ = (vaddr, asid);
        unimplemented!("sfence_vma_vaddr_asid is only available on RISC-V");
    }
}
