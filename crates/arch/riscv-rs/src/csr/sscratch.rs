//! Supervisor scratch register (`sscratch`, 0x140).

define_csr! {
    /// Supervisor scratch register (raw `u64`).
    Sscratch @ 0x140 {}
}
