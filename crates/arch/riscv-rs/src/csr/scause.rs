//! Supervisor cause register (`scause`, 0x142).

define_csr! {
    /// Supervisor cause register.
    Scause @ 0x142 {
        /// Exception code (bits `[62:0]`).
        field code: u64[0..=62],
        /// Interrupt bit (bit 63): `true` if the trap was an interrupt.
        field interrupt: bool[63..=63],
    }
}
