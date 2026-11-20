//! Supervisor address translation and protection (`satp`, 0x180).
//!
//! RV64 layout: `MODE[63:60] | ASID[59:44] | PPN[43:0]`.

/// Address-translation mode encoded in `satp.MODE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SatpMode {
    /// No translation / protection.
    Bare = 0,
    /// Page-based 39-bit virtual addressing.
    Sv39 = 8,
    /// Page-based 48-bit virtual addressing.
    Sv48 = 9,
    /// Page-based 57-bit virtual addressing.
    Sv57 = 10,
}

impl From<SatpMode> for u64 {
    #[inline(always)]
    fn from(value: SatpMode) -> Self {
        value as u64
    }
}

impl TryFrom<u64> for SatpMode {
    type Error = ();

    #[inline(always)]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bare),
            8 => Ok(Self::Sv39),
            9 => Ok(Self::Sv48),
            10 => Ok(Self::Sv57),
            _ => Err(()),
        }
    }
}

define_csr! {
    /// Supervisor address translation and protection register.
    Satp @ 0x180 {
        field ppn: u64[0..=43],
        field asid: u64[44..=59],
        field mode: SatpMode[60..=63],
    }
}
