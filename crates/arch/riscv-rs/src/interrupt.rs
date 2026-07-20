

/// Triggers a breakpoint.
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("ebreak");
    }
}

