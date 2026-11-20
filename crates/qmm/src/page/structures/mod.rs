use core::marker::PhantomData;

use crate::{KILOBYTE, PhysicalAddress, VirtualAddress};

pub mod table;
pub mod entry;
pub mod flags;
pub mod flush;
pub mod debug_tree;

pub use debug_tree::DebugTreeOpts;

pub struct PhysicalFrame<SIZE: PageSize = PageSize4KiB> {
    address: PhysicalAddress,
    phantom: PhantomData<SIZE>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageSize4KiB;

/// Trait for abstracting over the three possible page sizes on x86_64, 4KiB, 2MiB, 1GiB.
pub trait PageSize: Copy + Eq + PartialOrd + Ord { // TODO: Add Sealed trait
    /// The page size in bytes.
    const SIZE: u64;

    /// A string representation of the page size for debug output.
    const DEBUG_STR: &'static str;
}

/// This trait is implemented for 4KiB and 2MiB pages, but not for 1GiB pages.
pub trait NotGiantPageSize: PageSize {}

impl NotGiantPageSize for PageSize4KiB {}

impl PageSize for PageSize4KiB {
    const SIZE: u64 = 4 * KILOBYTE as u64;
    const DEBUG_STR: &'static str = "4KiB";
}

pub struct Page<SIZE: PageSize = PageSize4KiB> {
    start_address: VirtualAddress,
    phantom: PhantomData<SIZE>,
}

impl<SIZE: PageSize> Page<SIZE> {
    /// The page size in bytes.
    pub const SIZE: u64 = SIZE::SIZE;

    pub fn from_start_address(start_address: VirtualAddress) -> Self {
        if !start_address.is_aligned(Self::SIZE) {
            panic!("Start address is not aligned to the page size");
        }
        Self { start_address, phantom: PhantomData }
    }
}