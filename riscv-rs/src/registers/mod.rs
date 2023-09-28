pub mod satp;
mod sip;
pub mod sstatus;
#[macro_use]
pub mod macros;
/// General purpose registers
/// 
/// Avoid using these registers directly.
/// Compilers should manage these registers.
/// 
/// This module is used to set the `sp`, `gp` and `tp` registers
/// when entering the kernel.
pub mod general;

pub enum Action {
    Set,
    Clear,
}