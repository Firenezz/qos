//! Host-only field packing tests for CSR `Local` views.
//!
//! Run with: `cargo test -p riscv-rs --features std`

use riscv_rs::csr::satp::{self, SatpMode};
use riscv_rs::csr::scause;
use riscv_rs::csr::sstatus::{self, PrivilegeMode, Xlen};
use riscv_rs::csr::stvec::{self, StvecMode};

#[test]
fn sstatus_sie_sum_roundtrip() {
    let mut s = sstatus::Local::from_bits(0);
    s.set_sie(true);
    s.set_sum(true);
    assert!(s.sie());
    assert!(s.sum());
    assert_eq!(
        s.bits() & sstatus::Flags::SIE.bits(),
        sstatus::Flags::SIE.bits()
    );
    assert_eq!(
        s.bits() & sstatus::Flags::SUM.bits(),
        sstatus::Flags::SUM.bits()
    );
}

#[test]
fn sstatus_spp_and_uxl_canonical_bits() {
    let mut s = sstatus::Local::from_bits(0);
    s.set_spp(PrivilegeMode::Supervisor);
    s.set_uxl(Xlen::Xlen64);
    assert_eq!(s.spp(), PrivilegeMode::Supervisor);
    assert_eq!(s.uxl(), Xlen::Xlen64);
    assert_eq!(s.bits(), (1 << 8) | (2 << 32));
}

#[test]
fn sstatus_clears_preserve_other_fields() {
    let mut s = sstatus::Local::from_bits(
        sstatus::Flags::SIE.bits() | sstatus::Flags::SUM.bits() | sstatus::Flags::MXR.bits(),
    );
    s.set_sie(false);
    assert!(!s.sie());
    assert!(s.sum());
    assert!(s.mxr());
}

#[test]
fn stvec_mode_and_base_roundtrip() {
    let mut v = stvec::Local::from_bits(0);
    v.set_mode(StvecMode::Vectored);
    v.set_base(0x8020_0000);
    assert_eq!(v.mode(), StvecMode::Vectored);
    assert_eq!(v.base(), 0x8020_0000);
    assert_eq!(v.bits() & 0b11, 1);
    assert_eq!(v.bits() & !0b11, 0x8020_0000);
}

#[test]
fn scause_interrupt_and_code_roundtrip() {
    let mut c = scause::Local::from_bits(0);
    c.set_interrupt(true);
    c.set_code(9);
    assert!(c.interrupt());
    assert_eq!(c.code(), 9);
    assert_eq!(c.bits(), (1u64 << 63) | 9);
}

#[test]
fn satp_mode_asid_ppn_canonical_bits() {
    let mut satp = satp::Local::from_bits(0);
    satp.set_mode(SatpMode::Sv48);
    satp.set_asid(0xABCD);
    satp.set_ppn(0x234_5678_9ABC);
    assert_eq!(satp.mode(), SatpMode::Sv48);
    assert_eq!(satp.asid(), 0xABCD);
    assert_eq!(satp.ppn(), 0x234_5678_9ABC);
    assert_eq!(
        satp.bits(),
        (9u64 << 60) | (0xABCDu64 << 44) | 0x234_5678_9ABC
    );
}

#[test]
fn satp_bare_clears_mode() {
    let mut satp = satp::Local::from_bits(u64::MAX);
    satp.set_mode(SatpMode::Bare);
    assert_eq!(satp.mode(), SatpMode::Bare);
    assert_eq!(satp.bits() >> 60, 0);
}
