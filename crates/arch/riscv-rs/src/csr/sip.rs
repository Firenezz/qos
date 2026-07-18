//! Supervisor interrupt-pending register (`sip`, 0x144).

define_csr! {
    /// Supervisor interrupt-pending register.
    Sip @ 0x144 {
        flags Flags: u64 {
            /// Supervisor software interrupt pending.
            SSIP = 1 << 1,
            /// Supervisor timer interrupt pending.
            STIP = 1 << 5,
            /// Supervisor external interrupt pending.
            SEIP = 1 << 9,
        }
    }
}
