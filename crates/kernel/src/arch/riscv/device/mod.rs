pub mod cpu;

/// Probe and register boot-time arch devices (timers, IPI, …) with platform.
pub fn register_devices() {
    // TODO: Sstc/SBI timer, SSWI/SBI IPI backends
}
