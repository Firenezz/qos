use core::ops::{Add, Sub};

use crate::{TableKind, align_down_u64, align_up_u64};

pub mod range;

pub use range::{
    Address, AddressRange, AddressRangeLen, PhysicalAddressRange, VirtualAddressRange,
};
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn address(&self) -> usize {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    pub fn add(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }

    pub fn sub(&self, offset: usize) -> Self {
        Self(self.0 - offset)
    }
}

impl core::fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysicalAddress({:x})", self.0)
    }
}

impl core::fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<usize> for PhysicalAddress {
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    #[inline]
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn is_aligned(&self, alignment: impl Into<u64>) -> bool {
        self.is_aligned_u64(alignment.into())
    }

    #[inline]
    pub fn is_aligned_u64(&self, alignment: u64) -> bool {
        self.align_down_u64(alignment).as_usize() == self.0
    }

    #[inline]
    pub fn align_up(&self, alignment: impl Into<u64>) -> Self {
        Self(align_up_u64(self.0 as u64, alignment.into()) as usize)
    }

    #[inline]
    pub fn align_down(&self, alignment: impl Into<u64>) -> Self {
        self.align_down_u64(alignment.into())
    }

    #[inline]
    pub fn align_down_u64(&self, alignment: u64) -> Self {
        Self(align_down_u64(self.0 as u64, alignment) as usize)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self(ptr as *const () as usize)
    }

    #[inline]
    pub fn from_mut_ptr<T: ?Sized>(ptr: *mut T) -> Self {
        Self(ptr as *mut () as usize)
    }

    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub fn add(&self, offset: usize) -> Self {
        Self(self.0.checked_add(offset).unwrap_or_else(|| {
            panic!("VirtualAddress::add({:#x}, {:#x}) overflow", self.0, offset)
        }))
    }

    #[inline]
    pub fn sub(&self, offset: usize) -> Self {
        Self(self.0 - offset)
    }
}

impl Add<usize> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Add<VirtualAddress> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: VirtualAddress) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<VirtualAddress> for VirtualAddress {
    type Output = usize;

    fn sub(self, rhs: VirtualAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

/// A virtual address was not canonical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtAddrNotValid(pub usize);

impl VirtualAddress {
    /// Creates a virtual address without checking canonicity.
    ///
    /// Prefer [`Arch::try_virt`](crate::arch::Arch::try_virt) at API boundaries.
    ///
    /// # Safety
    ///
    /// `addr` must be canonical for the target architecture.
    #[inline]
    pub const unsafe fn new_unchecked(addr: usize) -> Self {
        Self(addr)
    }

    #[inline]
    pub const fn address(&self) -> usize {
        self.0
    }

    pub const fn kind(&self) -> TableKind {
        if (self.0 as isize) < 0 {
            TableKind::Kernel
        } else {
            TableKind::User
        }
    }
}

impl core::fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtualAddress({:#x})", self.0)
    }
}

impl core::fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryArea {
    pub base: PhysicalAddress,
    pub size: usize,
}

impl MemoryArea {
    pub fn new(base: PhysicalAddress, size: usize) -> Self {
        Self { base, size }
    }

    pub fn base(&self) -> PhysicalAddress {
        self.base
    }
}
