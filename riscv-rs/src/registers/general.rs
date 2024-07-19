use core::arch::asm;

macro_rules! implement_read_register {
    ($(#[$attr:meta])*, $fnname:ident, $asm:expr) => {
        pub fn $fnname() -> usize {
            /*let value: usize;
            unsafe {
                //asm!(concat!("mv {}, ", $asm), out(reg) value, options(nomem, nostack, preserves_flags));
            }
            value*/
            todo!()
        }
    };
}

macro_rules! implement_write_register {
    ($(#[$attr:meta])*, $fnname:ident, $asm:expr) => {
        pub fn $fnname(value: usize) {
            unsafe {
                //asm!(concat!("mv ", $asm, ", {}"), in(reg) value, options(nomem, nostack, preserves_flags));
            }
        }
    };
}

/// The `ra` register is used to hold the return address of a function call.
pub mod ra {
    use core::arch::asm;
    implement_read_register!(
            /// Read the value of the `ra` register
            , read, "ra");

    implement_write_register!(
            /// Write the value of the `ra` register
            , write, "ra");
}

/// The `sp` register is used to hold the stack pointer.
///
/// The stack pointer points to the lowest address of a memory region reserved for stack usage.
pub mod sp {
    use core::arch::asm;
    implement_read_register!(
            /// Read the value of the `sp` register
            , read, "sp");

    implement_write_register!(
            /// Write the value of the `sp` register
            , write, "sp");
}

/// The `gp` register is used to hold the global pointer.
///
/// The global pointer points to the lowest address of a memory region reserved for global data.
pub mod gp {
    use core::arch::asm;
    implement_read_register!(
            /// Read the value of the `gp` register
            , read, "gp");

    implement_write_register!(
            /// Write the value of the `gp` register
            , write, "gp");
}

/// The `tp` register is used to hold the thread pointer.
///
/// The thread pointer points to the lowest address of a memory region reserved for thread-local data.
pub mod tp {
    use core::arch::asm;
    implement_read_register!(
            /// Read the value of the `tp` register
            , read, "tp");

    implement_write_register!(
            /// Write the value of the `tp` register
            , write, "tp");
}

/// The `t0` register, also called `fp`, is used to hold a temporary value.
/// Also used as the frame pointer.
///
/// The frame pointer points to the lowest address of a memory region reserved for stack usage.
pub mod fp {
    use core::arch::asm;
    implement_read_register!(
            /// Read the value of the `t0` register
            , read, "fp");

    implement_write_register!(
            /// Write the value of the `t0` register
            , write, "fp");
}
