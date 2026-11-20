//! Match on `scause` and handle or halt.

use crate::arch::riscv::interrupt::exception::InterruptContext;

use super::cause::{Exception, Interrupt};

pub(super) fn handle(context: &mut InterruptContext) {
    let cause = riscv_rs::csr::scause::read();
    let is_interrupt = cause.interrupt();
    if !is_interrupt {
        match Exception::from_code(cause.code()) {
            Some(Exception::InstructionPageFault)
            | Some(Exception::LoadPageFault)
            | Some(Exception::StorePageFault) => {
                log::error!("Page fault: {:016X}", context.regs.ra);
                log::error!("Page fault stval: {:#016X}", riscv_rs::csr::stval::read().bits());
                unhandled(context, "page fault");
            }
            Some(Exception::UserEnvCall) | Some(Exception::SupervisorEnvCall) => {
                unhandled(context, "environment call");
            }
            Some(Exception::Breakpoint) => {
                unhandled(context, "breakpoint");
            }
            Some(_) | None => unhandled(context, "exception"),
        }
        
    } else {
        match Interrupt::from_code(cause.code()) {
            Some(Interrupt::SupervisorSoftware) => {
                unhandled(context, "supervisor software interrupt");
            }
            Some(Interrupt::SupervisorTimer) => {
                unhandled(context, "supervisor timer interrupt");
            }
            Some(Interrupt::SupervisorExternal) => {
                unhandled(context, "supervisor external interrupt");
            }
            None => unhandled(context, "unknown interrupt"),
        }
    }
}

/*pub(super) fn handle(frame: &mut TrapFrame) {
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
}*/

fn unhandled(context: &mut InterruptContext, kind: &str) {
    context.dump();
    log::error!("Unhandled interrupt: {}", kind);
    crate::halt();
}
