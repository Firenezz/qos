//! Shared CPU entry: one GP setup for ELF `e_entry` and Limine `EntryPointRequest`.

/// Kernel entry used by:
/// - linker `ENTRY(boot_main)` / ELF `e_entry`
/// - Limine `EntryPointRequest` (same symbol — no second entry stub)
///
/// Sets `gp`, then enters [`crate::boot::early_start`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn boot_main() -> ! {
    use core::arch::asm;
    // SAFETY: once per boot before any GP-relative Rust code.
    unsafe {
        asm!(
            ".option push",
            ".option norelax",
            "la gp, __global_pointer$",
            ".option pop",
            "tail {start}",
            start = sym crate::boot::kernel_main,
            options(nomem, nostack, noreturn),
        );
    }
}
