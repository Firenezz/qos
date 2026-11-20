//! Supervisor trap vector base address register (`stvec`, 0x105).

/// Trap vector mode encoded in `stvec[1:0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StvecMode {
    /// All exceptions set `pc` to `BASE`.
    Direct = 0,
    /// Asynchronous interrupts set `pc` to `BASE + 4 * cause`.
    Vectored = 1,
}

impl From<StvecMode> for u64 {
    #[inline(always)]
    fn from(value: StvecMode) -> Self {
        value as u64
    }
}

impl TryFrom<u64> for StvecMode {
    type Error = ();

    #[inline(always)]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Direct),
            1 => Ok(Self::Vectored),
            _ => Err(()),
        }
    }
}

define_csr! {
    /// Supervisor trap vector base-address register.
    Stvec @ 0x105 {
        field mode: StvecMode[0..=1],
        /// `BASE >> 2` (bits `[63:2]` of `stvec`).
        field base_ppn: u64[2..=63],
    }
}

impl crate::csr::Local<Stvec> {
    /// Aligned trap-vector base address (`BASE`).
    #[inline(always)]
    pub fn base(self) -> u64 {
        self.base_ppn() << 2
    }

    /// Set the trap-vector base address (low 2 bits of `addr` are ignored).
    #[inline(always)]
    pub fn set_base(&mut self, addr: u64) {
        self.set_base_ppn(addr >> 2);
    }
}
