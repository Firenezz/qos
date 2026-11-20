//! Arch-agnostic inter-processor interrupt service.
//!
//! Arch probes (e.g. ACLINT SSWI → SBI on RISC-V) and registers the winner.

/// Cross-hart signaling.
pub trait Ipi: Send + Sync {
    /// Request an IPI on `hart_id` (or arch-equivalent CPU id).
    fn send(&self, hart_id: usize);

    /// Local handler for the software / IPI interrupt.
    fn handle_interrupt(&self);

    /// Optional TLB shootdown helper; default sends IPI only.
    ///
    /// Arch backends may override with `remote_sfence_*` when available.
    fn remote_sfence_vma(&self, hart_id: usize, _start: usize, _size: usize) {
        self.send(hart_id);
    }
}
