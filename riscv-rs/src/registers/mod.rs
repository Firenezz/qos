//! RISC-V register definitions and operations
//!
//! This module provides safe abstractions for working with RISC-V registers,
//! including control and status registers (CSRs) and general-purpose registers.
//! The implementation uses compile-time optimizations to ensure maximum performance.

pub mod satp;
pub mod sip;
pub mod sstatus;
#[macro_use]
pub mod macros;

/// General purpose registers
/// 
/// This module provides access to general-purpose registers (GPRs).
/// Note that direct manipulation of these registers should be done with care,
/// as compilers typically manage their usage.
/// 
/// The module is primarily used for low-level operations such as:
/// - Setting up the stack pointer (`sp`)
/// - Initializing the global pointer (`gp`)
/// - Configuring the thread pointer (`tp`)
pub mod general;

/// Actions that can be performed on register bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Set the specified bits
    Set,
    /// Clear the specified bits
    Clear,
}

/// Trait for register operations with compile-time optimizations
pub trait Register {
    /// The type of the register value
    type Value;
    
    /// The CSR number for this register (if applicable)
    const CSR_NUMBER: Option<u16>;
    
    /// Read the current value of the register
    /// 
    /// This is implemented using inline assembly for maximum performance.
    /// The implementation is chosen at compile time based on the register type.
    fn read() -> Self::Value;
    
    /// Write a new value to the register
    /// 
    /// This is implemented using inline assembly for maximum performance.
    /// The implementation is chosen at compile time based on the register type.
    fn write(value: Self::Value);
    
    /// Modify specific bits in the register
    /// 
    /// This operation is atomic and uses the appropriate CSR instruction
    /// based on the operation type (set/clear).
    fn modify_bits<F>(f: F)
    where
        F: FnOnce(&mut Self::Value);
}

/// Trait for CSR-specific operations
pub trait CsrRegister: Register {
    /// Read the CSR and return its value
    /// 
    /// This uses the `csrr` instruction for maximum performance.
    fn csr_read() -> Self::Value;
    
    /// Write a value to the CSR
    /// 
    /// This uses the `csrw` instruction for maximum performance.
    fn csr_write(value: Self::Value);
    
    /// Set bits in the CSR
    /// 
    /// This uses the `csrs` instruction for maximum performance.
    fn csr_set_bits(mask: Self::Value);
    
    /// Clear bits in the CSR
    /// 
    /// This uses the `csrc` instruction for maximum performance.
    fn csr_clear_bits(mask: Self::Value);
}

/// Macro to define a CSR register with compile-time optimizations
#[macro_export]
macro_rules! define_csr {
    ($name:ident, $csr_number:expr, $value_type:ty) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        
        impl Register for $name {
            type Value = $value_type;
            const CSR_NUMBER: Option<u16> = Some($csr_number);
            
            #[inline(always)]
            fn read() -> Self::Value {
                let value: Self::Value;
                unsafe {
                    asm!(
                        "csrr {0}, {1}",
                        out(reg) value,
                        const $csr_number,
                        options(nostack, nomem)
                    );
                }
                value
            }
            
            #[inline(always)]
            fn write(value: Self::Value) {
                unsafe {
                    asm!(
                        "csrw {0}, {1}",
                        const $csr_number,
                        in(reg) value,
                        options(nostack, nomem)
                    );
                }
            }
            
            #[inline(always)]
            fn modify_bits<F>(f: F)
            where
                F: FnOnce(&mut Self::Value)
            {
                let mut value = Self::read();
                f(&mut value);
                Self::write(value);
            }
        }
        
        impl CsrRegister for $name {
            #[inline(always)]
            fn csr_read() -> Self::Value {
                Self::read()
            }
            
            #[inline(always)]
            fn csr_write(value: Self::Value) {
                Self::write(value)
            }
            
            #[inline(always)]
            fn csr_set_bits(mask: Self::Value) {
                unsafe {
                    asm!(
                        "csrs {0}, {1}",
                        const $csr_number,
                        in(reg) mask,
                        options(nostack, nomem)
                    );
                }
            }
            
            #[inline(always)]
            fn csr_clear_bits(mask: Self::Value) {
                unsafe {
                    asm!(
                        "csrc {0}, {1}",
                        const $csr_number,
                        in(reg) mask,
                        options(nostack, nomem)
                    );
                }
            }
        }
    };
}