//! Supervisor exception program counter (`sepc`, 0x141).

define_csr! {
    /// Supervisor exception program counter (raw PC).
    Sepc @ 0x141 {}
}
