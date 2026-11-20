//! `define_csr!` — generate CSR marker, asm accessors, and field helpers.
//!
//! Expand inside an existing CSR module file (e.g. `csr/sstatus.rs`).

/// Define CSR access for the current module.
///
/// # Example
///
/// ```ignore
/// define_csr! {
///     /// Supervisor status register marker
///     Sstatus @ 0x100 {
///         flags Flags: u64 {
///             SIE = 1 << 1,
///             SUM = 1 << 18,
///         }
///         field sie: bool[1..=1],
///         field spp: PrivilegeMode[8..=8],
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_csr {
    (
        $(#[$marker_meta:meta])*
        $marker:ident @ $addr:literal
        {
            $(
                flags $flags_name:ident : $flags_ty:ty {
                    $(
                        $(#[$($flag_attr:tt)*])*
                        $flag:ident = $flag_val:expr
                    ),+ $(,)?
                }
            )?
            $(
                $(#[$($field_attr:tt)*])*
                field $field_name:ident : $field_ty:ty [$field_lo:literal ..= $field_hi:literal]
            ),* $(,)?
        }
    ) => {
        $(#[$marker_meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $marker;

        impl $crate::csr::Csr for $marker {
            const ADDR: u16 = $addr;
            type Bits = u64;
        }

        impl $crate::csr::Readable for $marker {
            #[inline(always)]
            fn read_raw() -> u64 {
                #[cfg(target_arch = "riscv64")]
                {
                    let value: u64;
                    // SAFETY: CSR address is a compile-time constant from the privileged ISA.
                    unsafe {
                        core::arch::asm!(
                            "csrr {rd}, {csr}",
                            rd = out(reg) value,
                            csr = const $addr,
                            options(nomem, nostack, preserves_flags)
                        );
                    }
                    value
                }
                #[cfg(not(target_arch = "riscv64"))]
                {
                    unimplemented!("CSR read is only available on riscv64")
                }
            }
        }

        impl $crate::csr::Writeable for $marker {
            #[inline(always)]
            fn write_raw(value: u64) {
                #[cfg(target_arch = "riscv64")]
                {
                    // SAFETY: CSR address is a compile-time constant from the privileged ISA.
                    unsafe {
                        core::arch::asm!(
                            "csrw {csr}, {rs}",
                            csr = const $addr,
                            rs = in(reg) value,
                            options(nomem, nostack, preserves_flags)
                        );
                    }
                }
                #[cfg(not(target_arch = "riscv64"))]
                {
                    let _ = value;
                    unimplemented!("CSR write is only available on riscv64")
                }
            }

            #[inline(always)]
            fn set_bits(mask: u64) {
                #[cfg(target_arch = "riscv64")]
                {
                    // SAFETY: CSR address is a compile-time constant from the privileged ISA.
                    unsafe {
                        core::arch::asm!(
                            "csrrs x0, {csr}, {rs}",
                            csr = const $addr,
                            rs = in(reg) mask,
                            options(nomem, nostack, preserves_flags)
                        );
                    }
                }
                #[cfg(not(target_arch = "riscv64"))]
                {
                    let _ = mask;
                    unimplemented!("CSR set is only available on riscv64")
                }
            }

            #[inline(always)]
            fn clear_bits(mask: u64) {
                #[cfg(target_arch = "riscv64")]
                {
                    // SAFETY: CSR address is a compile-time constant from the privileged ISA.
                    unsafe {
                        core::arch::asm!(
                            "csrrc x0, {csr}, {rs}",
                            csr = const $addr,
                            rs = in(reg) mask,
                            options(nomem, nostack, preserves_flags)
                        );
                    }
                }
                #[cfg(not(target_arch = "riscv64"))]
                {
                    let _ = mask;
                    unimplemented!("CSR clear is only available on riscv64")
                }
            }
        }

        /// Local shadow of this CSR.
        pub type Local = $crate::csr::Local<$marker>;

        /// Read the CSR (`csrr`).
        #[inline(always)]
        pub fn read() -> Local {
            Local::read()
        }

        /// Write a full local value (`csrw`).
        #[inline(always)]
        pub fn write(value: Local) {
            value.commit();
        }

        /// Read raw bits.
        #[inline(always)]
        pub fn read_raw() -> u64 {
            <$marker as $crate::csr::Readable>::read_raw()
        }

        /// Write raw bits.
        #[inline(always)]
        pub fn write_raw(value: u64) {
            <$marker as $crate::csr::Writeable>::write_raw(value);
        }

        $(
            $crate::__define_csr_flags!($flags_name, $flags_ty, $($(#[$($flag_attr)*])* $flag = $flag_val),+);

            /// Atomically set bits (`csrrs`).
            #[inline(always)]
            pub fn set(flags: $flags_name) {
                <$marker as $crate::csr::Writeable>::set_bits(flags.bits());
            }

            /// Atomically clear bits (`csrrc`).
            #[inline(always)]
            pub fn clear(flags: $flags_name) {
                <$marker as $crate::csr::Writeable>::clear_bits(flags.bits());
            }
        )?

        $(
            $crate::__define_csr_field!($marker, $field_name, $field_ty, $field_lo, $field_hi);
        )*
    };
}

/// Internal: expand `bitflags!` inside a CSR module.
#[macro_export]
#[doc(hidden)]
macro_rules! __define_csr_flags {
    ($flags_name:ident, $flags_ty:ty, $($(#[$($flag_attr:tt)*])* $flag:ident = $flag_val:expr),+) => {
        bitflags::bitflags! {
            /// Bit mask for one-shot [`set`](set) / [`clear`](clear).
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $flags_name: $flags_ty {
                $(
                    $(#[$($flag_attr)*])*
                    const $flag = $flag_val;
                )+
            }
        }
    };
}

/// Internal: field getters/setters on [`Local`](crate::csr::Local).
#[macro_export]
#[doc(hidden)]
macro_rules! __define_csr_field {
    ($marker:ident, $field_name:ident, bool, $lo:literal, $hi:literal) => {
        $crate::__define_csr_field_bool!($marker, $field_name, $lo, $hi);
    };
    ($marker:ident, $field_name:ident, u64, $lo:literal, $hi:literal) => {
        $crate::__define_csr_field_u64!($marker, $field_name, $lo, $hi);
    };
    ($marker:ident, $field_name:ident, $field_ty:ty, $lo:literal, $hi:literal) => {
        $crate::__define_csr_field_enum!($marker, $field_name, $field_ty, $lo, $hi);
    };
}

/// Internal: boolean field.
#[macro_export]
#[doc(hidden)]
macro_rules! __define_csr_field_bool {
    ($marker:ident, $field_name:ident, $lo:literal, $hi:literal) => {
        ::paste::paste! {
            impl $crate::csr::Local<$marker> {
                /// Read this field from the local shadow.
                #[inline(always)]
                pub fn $field_name(self) -> bool {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const _: () = assert!(HI >= LO);
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = (1u64 << WIDTH) - 1;
                    ((self.bits() >> LO) & MASK) != 0
                }

                /// Set this field in the local shadow.
                #[inline(always)]
                pub fn [<set_ $field_name>](&mut self, value: bool) {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = ((1u64 << WIDTH) - 1) << LO;
                    let bits = self.bits();
                    *self.bits_mut() = if value {
                        bits | MASK
                    } else {
                        bits & !MASK
                    };
                }
            }
        }
    };
}

/// Internal: raw `u64` window field.
#[macro_export]
#[doc(hidden)]
macro_rules! __define_csr_field_u64 {
    ($marker:ident, $field_name:ident, $lo:literal, $hi:literal) => {
        ::paste::paste! {
            impl $crate::csr::Local<$marker> {
                /// Read this field from the local shadow.
                #[inline(always)]
                pub fn $field_name(self) -> u64 {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const _: () = assert!(HI >= LO);
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = if WIDTH == 64 {
                        u64::MAX
                    } else {
                        (1u64 << WIDTH) - 1
                    };
                    (self.bits() >> LO) & MASK
                }

                /// Set this field in the local shadow.
                #[inline(always)]
                pub fn [<set_ $field_name>](&mut self, value: u64) {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = if WIDTH == 64 {
                        u64::MAX
                    } else {
                        (1u64 << WIDTH) - 1
                    };
                    let bits = self.bits();
                    *self.bits_mut() = (bits & !(MASK << LO)) | ((value & MASK) << LO);
                }
            }
        }
    };
}

/// Internal: enum / typed field via `TryFrom<u64>` + `Into<u64>`.
#[macro_export]
#[doc(hidden)]
macro_rules! __define_csr_field_enum {
    ($marker:ident, $field_name:ident, $field_ty:ty, $lo:literal, $hi:literal) => {
        ::paste::paste! {
            impl $crate::csr::Local<$marker> {
                /// Read this field from the local shadow.
                #[inline(always)]
                pub fn $field_name(self) -> $field_ty {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const _: () = assert!(HI >= LO);
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = if WIDTH == 64 {
                        u64::MAX
                    } else {
                        (1u64 << WIDTH) - 1
                    };
                    let raw = (self.bits() >> LO) & MASK;
                    match <$field_ty as core::convert::TryFrom<u64>>::try_from(raw) {
                        Ok(v) => v,
                        Err(_) => panic!(concat!(
                            "invalid value for CSR field ",
                            stringify!($field_name)
                        )),
                    }
                }

                /// Set this field in the local shadow.
                #[inline(always)]
                pub fn [<set_ $field_name>](&mut self, value: $field_ty) {
                    const LO: u32 = $lo;
                    const HI: u32 = $hi;
                    const WIDTH: u32 = HI - LO + 1;
                    const MASK: u64 = if WIDTH == 64 {
                        u64::MAX
                    } else {
                        (1u64 << WIDTH) - 1
                    };
                    let encoded: u64 = core::convert::Into::<u64>::into(value);
                    let bits = self.bits();
                    *self.bits_mut() = (bits & !(MASK << LO)) | ((encoded & MASK) << LO);
                }
            }
        }
    };
}
