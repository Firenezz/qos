#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;
//#![warn(missing_docs)]

pub const KILOBYTE: usize = 1024;
pub const MEGABYTE: usize = KILOBYTE * 1024;
pub const GIGABYTE: usize = MEGABYTE * 1024;
#[cfg(target_pointer_width = "64")]
pub const TERABYTE: usize = GIGABYTE * 1024;

pub mod allocator;
pub mod arch;
pub mod hhdm;
pub mod page;
pub mod structures;
pub mod svpbmt;

use structures::*;

pub use hhdm::{hhdm_offset_or, set_hhdm_offset, try_hhdm_offset};
pub use svpbmt::{set_svpbmt_enabled, svpbmt_enabled};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableKind {
    User,
    Kernel,
}

pub enum TableType {
    Flat,
    Recursive,
}

/// Sign-extends `addr` to a canonical virtual address for the given width.
#[inline]
pub const fn sign_extend_virt(addr: usize, bits: u32) -> usize {
    #[cfg(target_pointer_width = "64")]
    {
        let shift = 64 - bits;
        ((addr << shift) as i64 >> shift) as usize
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let _ = bits;
        addr
    }
}

/// Align `addr` down to a power-of-two boundary.
#[inline]
pub const fn align_down_u64(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align `addr` up to a power-of-two boundary.
#[inline]
pub const fn align_up_u64(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let mask = align - 1;
    if addr & mask == 0 {
        addr
    } else if let Some(aligned) = (addr | mask).checked_add(1) {
        aligned
    } else {
        panic!("align_up overflow")
    }
}

/// Returns whether `addr` is canonical for the given virtual address width.
#[inline]
pub const fn is_canonical_virt(addr: usize, bits: u32) -> bool {
    sign_extend_virt(addr, bits) == addr
}

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::allocator::*;
    pub use crate::arch::riscv64::*;
    pub use crate::arch::*;
    pub use crate::hhdm::*;
    pub use crate::page::*;
    pub use crate::structures::*;
    pub use crate::svpbmt::*;
}
