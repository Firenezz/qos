//! Supervisor trap value register (`stval`, 0x143).

define_csr! {
    /// Supervisor trap value register (raw `u64`).
    Stval @ 0x143 {}
}
