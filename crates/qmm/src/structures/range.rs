use core::fmt;
use core::ops::Range;

use super::{PhysicalAddress, VirtualAddress};

/// Address types that can form half-open ranges `[start, end)`.
pub trait Address: Copy + PartialEq + PartialOrd + fmt::Display {
    fn as_usize(self) -> usize;

    fn from_usize(addr: usize) -> Self;

    fn saturating_add(self, offset: usize) -> Self;
}

impl Address for PhysicalAddress {
    fn as_usize(self) -> usize {
        self.address()
    }

    fn from_usize(addr: usize) -> Self {
        Self::new(addr)
    }

    fn saturating_add(self, offset: usize) -> Self {
        Self::new(self.address().saturating_add(offset))
    }
}

impl Address for VirtualAddress {
    fn as_usize(self) -> usize {
        self.address()
    }

    fn from_usize(addr: usize) -> Self {
        Self::new(addr)
    }

    fn saturating_add(self, offset: usize) -> Self {
        Self::new(self.as_usize().saturating_add(offset))
    }
}

/// Half-open address range `[start, end)`.
///
/// Interoperates with [`Range`] via [`From`]:
///
/// ```
/// use core::ops::Range;
/// use qmm::{PhysicalAddress, PhysicalAddressRange};
///
/// let range: Range<PhysicalAddress> = PhysicalAddressRange::from_base_len(
///     PhysicalAddress::new(0x1000),
///     0x1000,
/// )
/// .into();
/// assert_eq!(range.start, PhysicalAddress::new(0x1000));
/// assert_eq!(range.end, PhysicalAddress::new(0x2000));
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AddressRange<A: Address> {
    start: A,
    end: A,
}

pub type PhysicalAddressRange = AddressRange<PhysicalAddress>;
pub type VirtualAddressRange = AddressRange<VirtualAddress>;

impl PhysicalAddressRange {
    pub const EMPTY: Self = Self::new(PhysicalAddress(0), PhysicalAddress(0));
}

impl<A: Address> AddressRange<A> {
    pub const fn new(start: A, end: A) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self::new(A::from_usize(0), A::from_usize(0))
    }

    pub fn from_base_len(base: A, len: usize) -> Self {
        Self::new(base, base.saturating_add(len))
    }

    pub const fn start(&self) -> A {
        self.start
    }

    pub const fn end(&self) -> A {
        self.end
    }

    pub const fn base(&self) -> A {
        self.start()
    }

    pub fn len(&self) -> usize {
        self.end()
            .as_usize()
            .saturating_sub(self.start().as_usize())
    }

    pub fn length(&self) -> usize {
        self.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn end_inclusive(&self) -> A {
        A::from_usize(self.end().as_usize().saturating_sub(1))
    }

    pub fn contains(&self, address: A) -> bool {
        address >= self.start() && address < self.end()
    }

    pub fn overlaps(&self, other: Self) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }

    pub fn is_adjacent_to(&self, other: Self) -> bool {
        self.start() == other.end() || other.start() == self.end()
    }

    pub fn extend_len(self, extra: usize) -> Self {
        Self::new(self.start, self.end.saturating_add(extra))
    }

    pub fn prepend_len(self, extra: usize) -> Self {
        Self::new(
            A::from_usize(self.start.as_usize().saturating_sub(extra)),
            self.end,
        )
    }

    pub const fn as_range(&self) -> Range<A> {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}

/// Length for [`Range`] when its bounds are address types.
pub trait AddressRangeLen<A: Address> {
    fn len(&self) -> usize;
}

impl<A: Address> AddressRangeLen<A> for Range<A> {
    fn len(&self) -> usize {
        self.end
            .as_usize()
            .saturating_sub(self.start.as_usize())
    }
}

impl<A: Address> AddressRangeLen<A> for AddressRange<A> {
    fn len(&self) -> usize {
        AddressRange::len(self)
    }
}

impl<A: Address> fmt::Debug for AddressRange<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AddressRange({}..{})", self.start(), self.end())
    }
}

impl<A: Address> fmt::Display for AddressRange<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start(), self.end())
    }
}

impl<A: Address> From<Range<A>> for AddressRange<A> {
    fn from(range: Range<A>) -> Self {
        Self::new(range.start, range.end)
    }
}

impl<A: Address> From<AddressRange<A>> for Range<A> {
    fn from(value: AddressRange<A>) -> Self {
        value.as_range()
    }
}

impl From<super::MemoryArea> for PhysicalAddressRange {
    fn from(area: super::MemoryArea) -> Self {
        Self::from_base_len(area.base, area.size)
    }
}

impl From<PhysicalAddressRange> for super::MemoryArea {
    fn from(range: PhysicalAddressRange) -> Self {
        Self::new(range.base(), range.len())
    }
}
