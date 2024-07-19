// TODO: clean macros

#[macro_export]
macro_rules! read_csr {
    ($register:ident) => {
        #[inline]
        unsafe fn _read() -> usize {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => {
                    let r: usize;
                    core::arch::asm!(concat!("csrrs {0}, ", stringify!($register), ", x0"), out(reg) r, options(nomem, nostack, preserves_flags));
                    r
                }

                #[cfg(not(target_arch = "riscv64"))]
                () => unimplemented!(),
            }
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($register:ident) => {
        #[inline]
        unsafe fn _write(value: usize) {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => {
                    core::arch::asm!(concat!("csrw ", stringify!($register), ", {0}"), in(reg) value, options(nomem, nostack, preserves_flags));
                }

                #[cfg(not(target_arch = "riscv64"))]
                () => unimplemented!(),
            }
        }
    };
}

#[macro_export]
macro_rules! read_csr_as {
    ($register:ident) => {
        read_csr!($register);

        /// Reads the CSR
        #[inline]
        pub fn read() -> $register {
            $register {
                value: unsafe { Self::_read() },
            }
        }
    };
}

#[macro_export]
macro_rules! set {
    ($register:ident) => {
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _set(bits: usize) {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => core::arch::asm!(concat!("csrrs x0, ", stringify!($register), ", {0}"), in(reg) bits),

                #[cfg(not(target_arch = "riscv64"))]
                () => unimplemented!(),
            }
        }
    };
}

#[macro_export]
macro_rules! write_csr_usize {
    ($register:ident) => {
        write_csr!($register);

        /// Set the CSR
        #[inline]
        pub fn write(&self) {
            unsafe { Self::_write(self.value) }
        }
    };
}

#[macro_export]
macro_rules! clear {
    ($register:ident) => {
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _clear(bits: usize) {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => core::arch::asm!(concat!("csrrc x0, ", stringify!($register), ", {0}"), in(reg) bits),

                #[cfg(not(target_arch = "riscv64"))]
                () => unimplemented!(),
            }
        }
    };
}

#[macro_export]
macro_rules! set_clear {
    ($register:ident) => {
        set!($register);
        clear!($register);
    };
}

#[macro_export]
macro_rules! set_clear_csr {
    ($register:ident, $set_bit:ident, $clear_bit:ident, $bit:expr) => {
        /// Set the CSR
        #[inline]
        pub fn $set_bit() {
            unsafe { Self::_set($bit) }
        }

        /// Clear the CSR
        #[inline]
        pub fn $clear_bit() {
            unsafe { Self::_clear($bit) }
        }
    };
}

#[macro_export]
macro_rules! set_clear_csr_usize {
    ($register:ident, $set_fn_name:ident, $clear_fn_name:ident, $flags:ident) => {
        /// Set the CSR based on the flags
        pub fn $set_fn_name(flags: $flags) {
            unsafe { Self::_set(flags.bits()) }
        }

        /// Clear the CSR based on the flags
        pub fn $clear_fn_name(flags: $flags) {
            unsafe { Self::_clear(flags.bits()) }
        }
    };
}
