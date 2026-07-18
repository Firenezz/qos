//! Kernel definitions re-exported from portable [`qos_types`].
//!
//! Prefer `qos_types` for new code; these paths remain for existing call sites.

pub use qos_types::{memory, paging};

pub use memory::{
    align_down, align_up, PhysAddr, PhysAddrNotValid, VirtAddr, VirtAddrNotValid,
    ADDRESS_SPACE_SIZE, PHYS_ADDR_BITS,
};
