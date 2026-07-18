//! Supervisor status register (`sstatus`, 0x100).
//!
//! Bit indices follow the RISC-V privileged specification (RV64).

/// Previous privilege mode encoded in `sstatus.SPP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeMode {
    /// User mode.
    User = 0,
    /// Supervisor mode.
    Supervisor = 1,
}

impl From<PrivilegeMode> for u64 {
    #[inline(always)]
    fn from(value: PrivilegeMode) -> Self {
        value as u64
    }
}

impl TryFrom<u64> for PrivilegeMode {
    type Error = ();

    #[inline(always)]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::User),
            1 => Ok(Self::Supervisor),
            _ => Err(()),
        }
    }
}

/// UXLEN / SXLEN encoding in `sstatus.UXL` (and related fields).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Xlen {
    /// 32-bit effective register width.
    Xlen32 = 1,
    /// 64-bit effective register width.
    Xlen64 = 2,
}

impl From<Xlen> for u64 {
    #[inline(always)]
    fn from(value: Xlen) -> Self {
        value as u64
    }
}

impl TryFrom<u64> for Xlen {
    type Error = ();

    #[inline(always)]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Xlen32),
            2 => Ok(Self::Xlen64),
            _ => Err(()),
        }
    }
}

define_csr! {
    /// Supervisor status register.
    Sstatus @ 0x100 {
        flags Flags: u64 {
            /// Supervisor interrupt enable.
            SIE  = 1 << 1,
            /// Supervisor previous interrupt enable.
            SPIE = 1 << 5,
            /// U-mode big-endian.
            UBE  = 1 << 6,
            /// Supervisor previous privilege (also typed as [`PrivilegeMode`]).
            SPP  = 1 << 8,
            /// Permit supervisor user-memory access.
            SUM  = 1 << 18,
            /// Make executable pages readable.
            MXR  = 1 << 19,
        }
        field sie: bool[1..=1],
        field spie: bool[5..=5],
        field ube: bool[6..=6],
        field spp: PrivilegeMode[8..=8],
        field sum: bool[18..=18],
        field mxr: bool[19..=19],
        field uxl: Xlen[32..=33],
    }
}
