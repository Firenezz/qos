//! Match on `scause` and handle or halt.

use super::cause::{Exception, Interrupt};
use super::frame::TrapFrame;
use riscv_rs::csr::scause;

pub(super) fn handle(frame: &mut TrapFrame) {
    let cause = scause::Local::from_bits(frame.scause);
    let code = cause.code();

    if cause.interrupt() {
        match Interrupt::from_code(code) {
            Some(Interrupt::SupervisorSoftware) => {
                // IPI — not wired yet.
                unhandled(frame, "supervisor software interrupt");
            }
            Some(Interrupt::SupervisorTimer) => {
                // Clear/re-arm via SBI once timekeeping exists.
                unhandled(frame, "supervisor timer interrupt");
            }
            Some(Interrupt::SupervisorExternal) => {
                // PLIC claim — not wired yet.
                unhandled(frame, "supervisor external interrupt");
            }
            None => unhandled(frame, "unknown interrupt"),
        }
        return;
    }

    match Exception::from_code(code) {
        Some(Exception::UserEnvCall) | Some(Exception::SupervisorEnvCall) => {
            // Syscall path — skip ecall so a future handler can resume.
            // For now we halt; when syscalls exist, bump sepc by 4 here or there.
            unhandled(frame, "environment call");
        }
        Some(Exception::InstructionPageFault)
        | Some(Exception::LoadPageFault)
        | Some(Exception::StorePageFault) => {
            unhandled(frame, "page fault");
        }
        Some(Exception::Breakpoint) => {
            unhandled(frame, "breakpoint");
        }
        Some(_) | None => unhandled(frame, "exception"),
    }
}

fn unhandled(frame: &TrapFrame, kind: &str) {
    let _ = (frame, kind);
    // Park the hart; CSR fields in `frame` remain for debuggers.
    crate::halt();
}
