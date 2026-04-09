//! Supervisor Status Register (sstatus)
//!
//! The sstatus register is a CSR that contains information about the current state
//! of the processor in supervisor mode.

use bondrewd::{BitfieldEnum, Bitfields};
use crate::registers::{CsrRegister, Register};

// Supervisor Status Register number
const SSTATUS_CSR: u16 = 0x100;

/// Privilege mode for the previous privilege level
#[derive(BitfieldEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeMode {
    /// User mode
    User = 0,
    /// Supervisor mode
    Supervisor = 1,
}

/// Supervisor Status Register structure
#[derive(Bitfields, Debug, Clone, Copy, PartialEq, Eq)]
#[bondrewd(default_endianness = "be")]
pub struct Sstatus {
    /// Supervisor Interrupt Enable
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub sie: bool,
    /// Supervisor Previous Interrupt Enable
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub spie: bool,
    /// Supervisor Previous Privilege Mode
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub spp: PrivilegeMode,
    /// User Memory Access
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub sum: bool,
    /// Make eXecutable Readable
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub mxr: bool,
    /// Supervisor User Memory Access
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub sxl: bool,
    /// User Memory Access
    #[bondrewd(enum_primitive = "u8", bit_length = 1)]
    pub uxl: bool,
    /// Reserved bits
    #[bondrewd(bit_length = 25)]
    _reserved: u32,
}

define_csr!(Sstatus, SSTATUS_CSR, u64);

impl Sstatus {
    /// Enable supervisor interrupts
    #[inline(always)]
    pub fn enable_interrupts() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.sie = true;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Disable supervisor interrupts
    #[inline(always)]
    pub fn disable_interrupts() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.sie = false;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Check if supervisor interrupts are enabled
    #[inline(always)]
    pub fn interrupts_enabled() -> bool {
        let value = Self::read();
        let sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.sie
    }
    
    /// Set the previous privilege mode to user mode
    #[inline(always)]
    pub fn set_previous_user_mode() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.spp = PrivilegeMode::User;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Set the previous privilege mode to supervisor mode
    #[inline(always)]
    pub fn set_previous_supervisor_mode() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.spp = PrivilegeMode::Supervisor;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Enable user memory access in supervisor mode
    #[inline(always)]
    pub fn enable_user_memory_access() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.sum = true;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Disable user memory access in supervisor mode
    #[inline(always)]
    pub fn disable_user_memory_access() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.sum = false;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Enable make executable readable
    #[inline(always)]
    pub fn enable_mxr() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.mxr = true;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Disable make executable readable
    #[inline(always)]
    pub fn disable_mxr() {
        let mut value = Self::read();
        let mut sstatus = Sstatus::from_bytes(&value.to_le_bytes());
        sstatus.mxr = false;
        Self::write(u64::from_le_bytes(sstatus.into_bytes()));
    }
    
    /// Get the current privilege mode
    #[inline(always)]
    pub fn get_privilege_mode() -> PrivilegeMode {
        PrivilegeMode::Supervisor
    }
}

