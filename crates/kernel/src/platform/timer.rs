//! Arch-agnostic timer service.
//!
//! Arch code probes backends (e.g. Sstc → SBI on RISC-V), picks one, and
//! [`register`](crate::platform::register_timer)s it. Generic kernel code only
//! uses this trait.

/// System timer / deadline source.
pub trait Timer: Send + Sync {
    /// Monotonic counter reading (arch units, typically `time` CSR ticks).
    fn now(&self) -> u64;

    /// Program the next interrupt at an absolute counter value.
    fn set_deadline(&self, abs: u64);

    /// Ack / re-arm path from the timer interrupt handler.
    fn handle_interrupt(&self);

    /// Busy-wait helper; default spins on [`Self::now`].
    fn spin_until(&self, abs: u64) {
        while self.now() < abs {
            core::hint::spin_loop();
        }
    }
}
