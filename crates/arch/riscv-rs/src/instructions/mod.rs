//! RISC-V instruction set architecture support
//!
//! This module provides abstractions for working with RISC-V instructions,
//! including assembly operations and interrupt handling.

/// Assembly operations and inline assembly support
pub mod asm;

/// Interrupt and exception handling
pub mod interrupts;

/// Instruction encoding and decoding
pub mod encoding {
    //! Instruction encoding and decoding utilities
    //!
    //! This module provides functions for encoding and decoding RISC-V instructions,
    //! including support for different instruction formats and extensions.
}

/// Instruction set extensions
pub mod extensions {
    //! RISC-V instruction set extensions
    //!
    //! This module provides support for various RISC-V extensions,
    //! such as M (Multiplication), A (Atomic), F (Single-precision Floating-point),
    //! D (Double-precision Floating-point), etc.
}

use core::arch::asm;

