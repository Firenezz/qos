//! Supervisor Status Register (sstatus)
//!
//! The sstatus register is a CSR that contains information about the current state
//! of the processor in supervisor mode.

use bitflags::bitflags;
use crate::registers::{CsrRegister, Register};

// Supervisor Status Register number
const SSTATUS_CSR: u16 = 0x100;

bitflags! {
    /// Supervisor Status Register flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SstatusFlags: u64 {
        /// Supervisor Interrupt Enable
        const SIE = 1 << 1;
        /// Supervisor Previous Interrupt Enable
        const SPIE = 1 << 5;
        /// Supervisor Previous Privilege Mode
        const SPP = 1 << 8;
        /// User Memory Access
        const SUM = 1 << 18;
        /// Make eXecutable Readable
        const MXR = 1 << 19;
        /// Supervisor User Memory Access
        const SXL = 1 << 34;
        /// User Memory Access
        const UXL = 1 << 32;
    }
}

define_csr!(Sstatus, SSTATUS_CSR, u64);

impl Sstatus {
    /// Enable supervisor interrupts
    #[inline(always)]
    pub fn enable_interrupts() {
        Self::csr_set_bits(SstatusFlags::SIE.bits());
    }
    
    /// Disable supervisor interrupts
    #[inline(always)]
    pub fn disable_interrupts() {
        Self::csr_clear_bits(SstatusFlags::SIE.bits());
    }
    
    /// Check if supervisor interrupts are enabled
    #[inline(always)]
    pub fn interrupts_enabled() -> bool {
        (Self::read() & SstatusFlags::SIE.bits()) != 0
    }
    
    /// Set the previous privilege mode to user mode
    #[inline(always)]
    pub fn set_previous_user_mode() {
        Self::csr_clear_bits(SstatusFlags::SPP.bits());
    }
    
    /// Set the previous privilege mode to supervisor mode
    #[inline(always)]
    pub fn set_previous_supervisor_mode() {
        Self::csr_set_bits(SstatusFlags::SPP.bits());
    }
    
    /// Enable user memory access in supervisor mode
    #[inline(always)]
    pub fn enable_user_memory_access() {
        Self::csr_set_bits(SstatusFlags::SUM.bits());
    }
    
    /// Disable user memory access in supervisor mode
    #[inline(always)]
    pub fn disable_user_memory_access() {
        Self::csr_clear_bits(SstatusFlags::SUM.bits());
    }
    
    /// Enable make executable readable
    #[inline(always)]
    pub fn enable_mxr() {
        Self::csr_set_bits(SstatusFlags::MXR.bits());
    }
    
    /// Disable make executable readable
    #[inline(always)]
    pub fn disable_mxr() {
        Self::csr_clear_bits(SstatusFlags::MXR.bits());
    }
}

