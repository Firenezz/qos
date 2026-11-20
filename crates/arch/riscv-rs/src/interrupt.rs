/// Triggers a breakpoint.
pub fn breakpoint() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("ebreak");
    }
}
