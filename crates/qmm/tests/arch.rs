#![cfg(target_pointer_width = "64")]

use qmm::arch::riscv64::Sv48Arch;
use qmm::arch::{Arch, EmulateArch};
use qmm::page::structures::flags::PageFlags;
use qmm::{VirtAddrNotValid, VirtualAddress, is_canonical_virt, sign_extend_virt};

const VA_BITS: u32 = 48;

#[test]
fn sign_extend_preserves_canonical_addresses() {
    assert_eq!(sign_extend_virt(0, VA_BITS), 0);
    assert_eq!(sign_extend_virt(0x1000, VA_BITS), 0x1000);
    assert_eq!(
        sign_extend_virt(0xFFFF_8000_0000_0000, VA_BITS),
        0xFFFF_8000_0000_0000
    );
}

#[test]
fn sign_extend_fixes_non_canonical_addresses() {
    assert_eq!(
        sign_extend_virt(0x0000_8000_0000_0000, VA_BITS),
        0xFFFF_8000_0000_0000
    );
    assert_eq!(
        sign_extend_virt(0x0000_FFFF_FFFF_FFFF, VA_BITS),
        0xFFFF_FFFF_FFFF_FFFF
    );
}

#[test]
fn is_canonical_detects_hole_addresses() {
    assert!(is_canonical_virt(0, VA_BITS));
    assert!(is_canonical_virt(0xFFFF_8000_0000_0000, VA_BITS));
    assert!(!is_canonical_virt(0x0000_8000_0000_0000, VA_BITS));
    assert!(!is_canonical_virt(0x0000_FFFF_FFFF_FFFF, VA_BITS));
}

#[test]
fn virtual_address_alignment() {
    let addr = VirtualAddress::new(0x1001);
    assert!(!addr.is_aligned(0x1000_u64));
    assert_eq!(addr.align_down(0x1000_u64).as_usize(), 0x1000);
    assert_eq!(addr.align_up(0x1000_u64).as_usize(), 0x2000);

    let aligned = VirtualAddress::new(0x1000);
    assert!(aligned.is_aligned(0x1000_u64));
    assert_eq!(aligned.align_up(0x1000_u64).as_usize(), 0x1000);
    assert_eq!(aligned.align_down(0x1000_u64).as_usize(), 0x1000);
}

#[test]
fn sv48_arch_is_canonical() {
    fn yes(address: usize) {
        let virt = unsafe { VirtualAddress::new_unchecked(address) };
        assert!(
            Sv48Arch::virt_is_valid(virt),
            "{address:#x} should be canonical"
        );
        assert!(
            Sv48Arch::try_virt(address).is_ok(),
            "{address:#x} should construct via try_virt"
        );
    }

    fn no(address: usize) {
        let virt = unsafe { VirtualAddress::new_unchecked(address) };
        assert!(
            !Sv48Arch::virt_is_valid(virt),
            "{address:#x} should not be canonical"
        );
        assert!(
            Sv48Arch::try_virt(address).is_err(),
            "{address:#x} should be rejected by try_virt"
        );
    }

    // Close to identical when compared to x86_64.
    yes(0xFFFF_8000_1337_1337);
    yes(0xFFFF_FFFF_FFFF_FFFF);
    yes(0x0000_0000_0000_0042);
    yes(0x0000_7FFF_FFFF_FFFF);
    no(0x1337_0000_0000_0000);
    no(0x1337_8000_0000_0000);
    no(0x0000_8000_0000_0000);
}

#[test]
fn emulate_arch_page_address_shift_is_48() {
    assert_eq!(EmulateArch::PAGE_ADDRESS_SHIFT, 48);
}

#[test]
fn emulate_arch_try_virt() {
    assert!(EmulateArch::try_virt(0x1000).is_ok());
    assert!(EmulateArch::try_virt(0xFFFF_8000_0000_0000).is_ok());

    assert_eq!(
        EmulateArch::try_virt(0x0000_8000_0000_0000),
        Err(VirtAddrNotValid(0x0000_8000_0000_0000))
    );
}

#[test]
fn emulate_arch_canonicalize_virt() {
    let virt = EmulateArch::canonicalize_virt(0x0000_8000_0000_0000);
    assert_eq!(virt.address(), 0xFFFF_8000_0000_0000);
}

#[test]
fn emulate_arch_virt_is_valid() {
    let valid = unsafe { VirtualAddress::new_unchecked(0xFFFF_8000_0000_0000) };
    let invalid = unsafe { VirtualAddress::new_unchecked(0x0000_8000_0000_0000) };

    assert!(EmulateArch::virt_is_valid(valid));
    assert!(!EmulateArch::virt_is_valid(invalid));
}

#[test]
fn runtime_hhdm_offset_overrides_default() {
    qmm::set_hhdm_offset(0xFFFF_C000_0000_0000);

    let phys = qmm::PhysicalAddress::new(0x1000);
    let virt = EmulateArch::phys_to_virt(phys);
    assert_eq!(virt.address(), 0xFFFF_C000_0000_1000);

    // Clear so other tests fall back to DEFAULT_HHDM_OFFSET.
    qmm::set_hhdm_offset(0);
}

#[test]
fn emulate_arch_phys_to_virt() {
    qmm::set_hhdm_offset(0);
    let phys = qmm::PhysicalAddress::new(0x2000);
    let virt = EmulateArch::phys_to_virt(phys);
    assert_eq!(virt.address(), EmulateArch::DEFAULT_HHDM_OFFSET + 0x2000);
    assert!(EmulateArch::virt_is_valid(virt));
}

#[test]
fn virtual_address_kind() {
    let user = unsafe { VirtualAddress::new_unchecked(0x1000) };
    let kernel = unsafe { VirtualAddress::new_unchecked(0xFFFF_8000_0000_0000) };

    assert_eq!(user.kind(), qmm::TableKind::User);
    assert_eq!(kernel.kind(), qmm::TableKind::Kernel);
}

#[test]
fn emulate_arch_read_write() {
    EmulateArch::init();

    let phys = qmm::PhysicalAddress::new(0x3000);
    let virt = EmulateArch::phys_to_virt(phys);

    unsafe {
        EmulateArch::write(virt, 0xDEAD_BEEF_u32);
        assert_eq!(EmulateArch::read::<u32>(virt), 0xDEAD_BEEF);
    }
}

#[test]
fn emulate_arch_write_bytes() {
    EmulateArch::init();

    let virt = EmulateArch::phys_to_virt(qmm::PhysicalAddress::new(0x4000));

    unsafe {
        EmulateArch::write_bytes(virt, 0xAB, 4);
        assert_eq!(EmulateArch::read::<u32>(virt), 0xABAB_ABAB);
    }
}

#[test]
fn emulate_arch_table_roundtrip() {
    EmulateArch::init();

    let table = qmm::PhysicalAddress::new(0x5000);
    unsafe {
        EmulateArch::set_table(qmm::TableKind::Kernel, table);
    }
    assert_eq!(EmulateArch::table(qmm::TableKind::Kernel), table);
}

#[test]
fn sv48_pbmt_constants() {
    assert_eq!(Sv48Arch::ENTRY_PBMT_MASK, 0b11 << 61);
    assert_eq!(Sv48Arch::ENTRY_FLAG_UNCACHEABLE, 1 << 61);
    assert_eq!(Sv48Arch::ENTRY_FLAG_DEVICE_MEMORY, 1 << 62);
    assert_eq!(
        Sv48Arch::ENTRY_FLAG_WRITE_COMBINING,
        Sv48Arch::ENTRY_FLAG_UNCACHEABLE
    );
}

#[test]
fn sv48_page_flags_pbmt_requires_svpbmt() {
    qmm::set_svpbmt_enabled(false);
    let flags = PageFlags::<Sv48Arch>::new().uncacheable(true);
    assert_eq!(flags.pbmt(), 0);

    qmm::set_svpbmt_enabled(true);
    let flags = PageFlags::<Sv48Arch>::new().uncacheable(true);
    assert_eq!(flags.pbmt(), 1 << 61);
    assert!(flags.has_uncacheable());

    let flags = PageFlags::<Sv48Arch>::new()
        .uncacheable(true)
        .device_memory(true);
    assert!(flags.has_device_memory());
    assert!(!flags.has_uncacheable());

    qmm::set_svpbmt_enabled(false);
}

#[test]
fn sv48_page_flags_write_combining_uses_nc() {
    qmm::set_svpbmt_enabled(true);
    let flags = PageFlags::<Sv48Arch>::new().write_combining(true);
    assert!(flags.has_write_combining());
    assert!(flags.has_uncacheable());
    qmm::set_svpbmt_enabled(false);
}

#[test]
fn emulate_arch_init_exposes_memory() {
    let areas = EmulateArch::init();
    assert_eq!(areas.len(), 1);
    assert_eq!(areas[0].base().address(), 0);
    assert!(areas[0].size > 0);
}
