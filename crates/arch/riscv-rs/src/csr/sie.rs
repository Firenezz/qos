//! Supervisor interrupt-enable register (`sie`, 0x104).

define_csr! {
    /// Supervisor interrupt-enable register.
    Sie @ 0x104 {
        flags Flags: u64 {
            /// Supervisor software interrupt enable.
            SSIE = 1 << 1,
            /// Supervisor timer interrupt enable.
            STIE = 1 << 5,
            /// Supervisor external interrupt enable.
            SEIE = 1 << 9,
        }
    }
}
