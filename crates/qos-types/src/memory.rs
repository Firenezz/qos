//! Physical and virtual address types for `riscv64` (Sv48).

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Size of the Sv48 usable virtual address space (`2^48`).
pub const ADDRESS_SPACE_SIZE: u64 = 0x1_0000_0000_0000;

/// Number of physical address bits in Sv39/Sv48/Sv57.
pub const PHYS_ADDR_BITS: u32 = 56;

/// Mask of valid physical address bits.
const PHYS_ADDR_MASK: u64 = (1 << PHYS_ADDR_BITS) - 1;

/// A canonical 64-bit virtual memory address.
///
/// This is a wrapper type around an `u64`, so it is always 8 bytes, even when compiled
/// on non 64-bit systems. The
/// [`TryFrom`](https://doc.rust-lang.org/core/convert/trait.TryFrom.html) trait can be used for performing conversions
/// between `u64` and `usize`.
///
/// On `x86_64`, only the 48 lower bits of a virtual address can be used. The top 16 bits need
/// to be copies of bit 47, i.e. the most significant bit. Addresses that fulfil this criterion
/// are called “canonical”. This type guarantees that it always represents a canonical address.
///
/// On `riscv64` with Sv48, only the 48 lower bits of a virtual address can be used. Bits 63–48
/// must all equal bit 47, or else a page-fault exception occurs. Addresses that fulfil this
/// criterion are called “canonical”. This type guarantees that it always represents a canonical
/// address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

/// A 64-bit physical memory address.
///
/// This is a wrapper type around an `u64`, so it is always 8 bytes, even when compiled
/// on non 64-bit systems. The
/// [`TryFrom`](https://doc.rust-lang.org/core/convert/trait.TryFrom.html) trait can be used for performing conversions
/// between `u64` and `usize`.
///
/// On `x86_64`, only the 52 lower bits of a physical address can be used. The top 12 bits need
/// to be 0.
///
/// On `riscv64` (Sv39/Sv48/Sv57), only the 56 lower bits of a physical address can be used.
/// Bits 63–56 must be 0. This type guarantees that it always represents a valid physical
/// address under that constraint.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

/// A passed `u64` was not a valid virtual address.
///
/// This means that bits 48 to 64 are not a valid sign extension of bit 47. Automatic sign
/// extension would have overwritten possibly meaningful bits. This likely indicates a bug,
/// for example an invalid address calculation.
///
/// Contains the invalid address.
pub struct VirtAddrNotValid(pub u64);

impl fmt::Debug for VirtAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

/// A passed `u64` was not a valid physical address.
///
/// This means that one or more bits in the range 56 to 64 were set.
///
/// Contains the invalid address.
pub struct PhysAddrNotValid(pub u64);

impl fmt::Debug for PhysAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl VirtAddr {
    /// Creates a new canonical virtual address.
    ///
    /// The provided address should already be canonical. If you want to check whether an
    /// address is canonical, use [`try_new`](Self::try_new).
    ///
    /// # Panics
    ///
    /// Panics if bits 48 to 64 are not a proper sign extension of bit 47.
    #[inline]
    pub const fn new(addr: u64) -> VirtAddr {
        match Self::try_new(addr) {
            Ok(v) => v,
            Err(_) => panic!("virtual address must be sign extended in bits 48 to 64"),
        }
    }

    /// Tries to create a new canonical virtual address.
    ///
    /// An address is canonical if bits 48 to 64 are a correct sign extension of bit 47.
    #[inline]
    pub const fn try_new(addr: u64) -> Result<VirtAddr, VirtAddrNotValid> {
        let v = Self::new_truncate(addr);
        if v.0 == addr {
            Ok(v)
        } else {
            Err(VirtAddrNotValid(addr))
        }
    }

    /// Creates a new canonical virtual address, discarding bits 48..64.
    ///
    /// Sign-extends bit 47 to make the address canonical, overwriting bits 48 to 64.
    #[inline]
    pub const fn new_truncate(addr: u64) -> VirtAddr {
        // Right-shifting as `i64` sign-extends, repeating the leftmost bit.
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    /// Creates a new virtual address without checking canonicity.
    ///
    /// # Safety
    ///
    /// Bits 48..64 must equal bit 47.
    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> VirtAddr {
        VirtAddr(addr)
    }

    /// Creates a virtual address that points to `0`.
    #[inline]
    pub const fn zero() -> VirtAddr {
        VirtAddr(0)
    }

    /// Converts the address to a `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Creates a virtual address from the given pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr.cast::<()>() as u64)
    }

    /// Converts the address to a raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    /// Converts the address to a mutable raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    /// Returns `true` if the address is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See [`align_up`] for more information.
    ///
    /// # Panics
    ///
    /// Panics if the resulting address overflows or is non-canonical.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VirtAddr::new(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See [`align_down`] for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VirtAddr::new_truncate(align_down(self.0, align.into()))
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.align_down(align).as_u64() == self.as_u64()
    }

    /// Returns the 12-bit page offset of this virtual address.
    #[inline]
    pub const fn page_offset(self) -> u16 {
        (self.0 & 0xfff) as u16
    }

    /// Returns the 9-bit VPN\[0\] (Sv48 level-0 / 4 KiB PTE) index.
    #[inline]
    pub const fn vpn0_index(self) -> u16 {
        ((self.0 >> 12) & 0x1ff) as u16
    }

    /// Returns the 9-bit VPN\[1\] (Sv48 level-1 / 2 MiB) index.
    #[inline]
    pub const fn vpn1_index(self) -> u16 {
        ((self.0 >> 21) & 0x1ff) as u16
    }

    /// Returns the 9-bit VPN\[2\] (Sv48 level-2 / 1 GiB) index.
    #[inline]
    pub const fn vpn2_index(self) -> u16 {
        ((self.0 >> 30) & 0x1ff) as u16
    }

    /// Returns the 9-bit VPN\[3\] (Sv48 level-3 / 512 GiB) index.
    #[inline]
    pub const fn vpn3_index(self) -> u16 {
        ((self.0 >> 39) & 0x1ff) as u16
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    /// Adds an offset to a virtual address.
    ///
    /// # Panics
    ///
    /// Panics on overflow or if the result is not canonical.
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::try_new(
            self.0
                .checked_add(rhs)
                .expect("attempt to add with overflow"),
        )
        .expect("attempt to add resulted in non-canonical virtual address")
    }
}

impl AddAssign<u64> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    /// Subtracts an offset from a virtual address.
    ///
    /// # Panics
    ///
    /// Panics on overflow or if the result is not canonical.
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::try_new(
            self.0
                .checked_sub(rhs)
                .expect("attempt to subtract with overflow"),
        )
        .expect("attempt to subtract resulted in non-canonical virtual address")
    }
}

impl SubAssign<u64> for VirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = u64;

    /// Returns the difference between two addresses.
    ///
    /// # Panics
    ///
    /// Panics on overflow.
    #[inline]
    fn sub(self, rhs: VirtAddr) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

impl TryFrom<u64> for VirtAddr {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(addr: u64) -> Result<Self, Self::Error> {
        Self::try_new(addr)
    }
}

impl From<VirtAddr> for u64 {
    #[inline]
    fn from(addr: VirtAddr) -> Self {
        addr.as_u64()
    }
}

#[cfg(target_pointer_width = "64")]
impl TryFrom<usize> for VirtAddr {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(addr: usize) -> Result<Self, Self::Error> {
        Self::try_new(addr as u64)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<VirtAddr> for usize {
    #[inline]
    fn from(addr: VirtAddr) -> Self {
        addr.as_u64() as usize
    }
}

impl PhysAddr {
    /// Creates a new physical address.
    ///
    /// # Panics
    ///
    /// Panics if any bit in the range 56 to 64 is set.
    #[inline]
    pub const fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(p) => p,
            Err(_) => panic!("physical addresses must not have any bits in the range 56 to 64 set"),
        }
    }

    /// Creates a new physical address, discarding bits 56..64.
    #[inline]
    pub const fn new_truncate(addr: u64) -> PhysAddr {
        PhysAddr(addr & PHYS_ADDR_MASK)
    }

    /// Creates a new physical address without checking validity.
    ///
    /// # Safety
    ///
    /// Bits 56..64 must be zero.
    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }

    /// Tries to create a new physical address.
    ///
    /// Fails if any bits in the range 56 to 64 are set.
    #[inline]
    pub const fn try_new(addr: u64) -> Result<Self, PhysAddrNotValid> {
        let p = Self::new_truncate(addr);
        if p.0 == addr {
            Ok(p)
        } else {
            Err(PhysAddrNotValid(addr))
        }
    }

    /// Creates a physical address that points to `0`.
    #[inline]
    pub const fn zero() -> PhysAddr {
        PhysAddr(0)
    }

    /// Converts the address to a `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if the address is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the physical address upwards to the given alignment.
    ///
    /// See [`align_up`] for more information.
    ///
    /// # Panics
    ///
    /// Panics if the resulting address has a bit in the range 56 to 64 set.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PhysAddr::new(align_up(self.0, align.into()))
    }

    /// Aligns the physical address downwards to the given alignment.
    ///
    /// See [`align_down`] for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PhysAddr(align_down(self.0, align.into()))
    }

    /// Checks whether the physical address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.align_down(align).as_u64() == self.as_u64()
    }

    /// Returns the 12-bit page offset of this physical address.
    #[inline]
    pub const fn page_offset(self) -> u16 {
        (self.0 & 0xfff) as u16
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_add(rhs).expect("attempt to add with overflow"))
    }
}

impl AddAssign<u64> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(
            self.0
                .checked_sub(rhs)
                .expect("attempt to subtract with overflow"),
        )
    }
}

impl SubAssign<u64> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = u64;

    #[inline]
    fn sub(self, rhs: PhysAddr) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

impl TryFrom<u64> for PhysAddr {
    type Error = PhysAddrNotValid;

    #[inline]
    fn try_from(addr: u64) -> Result<Self, Self::Error> {
        Self::try_new(addr)
    }
}

impl From<PhysAddr> for u64 {
    #[inline]
    fn from(addr: PhysAddr) -> Self {
        addr.as_u64()
    }
}

#[cfg(target_pointer_width = "64")]
impl TryFrom<usize> for PhysAddr {
    type Error = PhysAddrNotValid;

    #[inline]
    fn try_from(addr: usize) -> Result<Self, Self::Error> {
        Self::try_new(addr as u64)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<PhysAddr> for usize {
    #[inline]
    fn from(addr: PhysAddr) -> Self {
        addr.as_u64() as usize
    }
}

/// Align address downwards.
///
/// Returns the greatest `x` with alignment `align` such that `x <= addr`.
///
/// # Panics
///
/// Panics if the alignment is not a power of two.
#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest `x` with alignment `align` such that `x >= addr`.
///
/// # Panics
///
/// Panics if the alignment is not a power of two or if an overflow occurs.
#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else if let Some(aligned) = (addr | align_mask).checked_add(1) {
        aligned
    } else {
        panic!("attempt to add with overflow")
    }
}
