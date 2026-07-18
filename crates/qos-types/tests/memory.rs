//! Host-only tests for address types and alignment helpers.
//!
//! Run with: `cargo test -p qos-types --features std`

use qos_types::{align_down, align_up, PhysAddr, VirtAddr, PHYS_ADDR_BITS};

const PHYS_ADDR_MASK: u64 = (1 << PHYS_ADDR_BITS) - 1;

#[test]
fn virtaddr_new_truncate() {
    assert_eq!(VirtAddr::new_truncate(0).as_u64(), 0);
    assert_eq!(
        VirtAddr::new_truncate(1 << 47).as_u64(),
        0xffff_8000_0000_0000
    );
    assert_eq!(VirtAddr::new_truncate(123).as_u64(), 123);
}

#[test]
fn virtaddr_try_new() {
    assert!(VirtAddr::try_new(0).is_ok());
    assert!(VirtAddr::try_new(0xffff_8000_0000_0000).is_ok());
    assert!(VirtAddr::try_new(1 << 47).is_err());
    assert!(VirtAddr::try_new(0x0001_0000_0000_0000).is_err());
}

#[test]
fn physaddr_truncate_and_try() {
    assert_eq!(PhysAddr::new_truncate(0x10).as_u64(), 0x10);
    assert_eq!(
        PhysAddr::new_truncate(1 << 56).as_u64(),
        0,
        "bit 56 must be discarded"
    );
    assert!(PhysAddr::try_new(PHYS_ADDR_MASK).is_ok());
    assert!(PhysAddr::try_new(1 << 56).is_err());
}

#[test]
fn virtaddr_vpn_indices() {
    // VA = VPN3 | VPN2 | VPN1 | VPN0 | offset
    let addr = VirtAddr::new(
        (0x12u64 << 39) | (0x34 << 30) | (0x56 << 21) | (0x78 << 12) | 0xabc,
    );
    assert_eq!(addr.page_offset(), 0xabc);
    assert_eq!(addr.vpn0_index(), 0x78);
    assert_eq!(addr.vpn1_index(), 0x56);
    assert_eq!(addr.vpn2_index(), 0x34);
    assert_eq!(addr.vpn3_index(), 0x12);
}

#[test]
fn align() {
    assert_eq!(align_down(0x1234, 0x1000), 0x1000);
    assert_eq!(align_up(0x1001, 0x1000), 0x2000);
    assert_eq!(align_up(0x1000, 0x1000), 0x1000);
}

#[test]
#[should_panic]
fn add_overflow_virtaddr() {
    let _ = VirtAddr::new(0xffff_ffff_ffff_ffff) + 1;
}

#[test]
#[should_panic]
fn add_overflow_physaddr() {
    let _ = PhysAddr::new(PHYS_ADDR_MASK) + 1;
}
