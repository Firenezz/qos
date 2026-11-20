#[derive(Debug, Copy, Clone)]
#[repr(C)]
/// Registers saved on trap entry — a view of the stack frame from [`push_registers!`].
///
/// Layout matches ABI names for `x1`–`x31` at slots `0`–`30` (`sepc` is slot 31,
/// outside this struct). `sp` and `tp` may be filled by the entry path rather than
/// the push macro.
pub struct InterruptRegisters {
    /// Return address
    pub ra: u64,  // x1
    /// Stack pointer
    pub sp: u64,  // x2
    /// Global pointer
    pub gp: u64,  // x3
    /// Thread pointer
    pub tp: u64,  // x4
    /// Temporary register 0
    pub t0: u64,  // x5
    pub t1: u64,  // x6
    pub t2: u64,  // x7
    /// Frame pointer and also s0
    pub fp: u64,  // x8  (s0)
    /// Saved register 1
    pub s1: u64,  // x9
    /// Argument 0
    pub a0: u64,  // x10
    pub a1: u64,  // x11
    pub a2: u64,  // x12
    pub a3: u64,  // x13
    pub a4: u64,  // x14
    pub a5: u64,  // x15
    pub a6: u64,  // x16
    pub a7: u64,  // x17
    pub s2: u64,  // x18
    pub s3: u64,  // x19
    pub s4: u64,  // x20
    pub s5: u64,  // x21
    pub s6: u64,  // x22
    pub s7: u64,  // x23
    pub s8: u64,  // x24
    pub s9: u64,  // x25
    pub s10: u64, // x26
    pub s11: u64, // x27
    pub t3: u64,  // x28
    pub t4: u64,  // x29
    pub t5: u64,  // x30
    pub t6: u64,  // x31
}

impl InterruptRegisters {
    pub fn dump(&self) {
        log::debug!(
            "InterruptRegisters:\n\
             ra:  {:016X}\n\
             sp:  {:016X}\n\
             gp:  {:016X}\n\
             tp:  {:016X}\n\
             t0:  {:016X}\n\
             t1:  {:016X}\n\
             t2:  {:016X}\n\
             s0:  {:016X}\n\
             s1:  {:016X}\n\
             a0:  {:016X}\n\
             a1:  {:016X}\n\
             a2:  {:016X}\n\
             a3:  {:016X}\n\
             a4:  {:016X}\n\
             a5:  {:016X}\n\
             a6:  {:016X}\n\
             a7:  {:016X}\n\
             s2:  {:016X}\n\
             s3:  {:016X}\n\
             s4:  {:016X}\n\
             s5:  {:016X}\n\
             s6:  {:016X}\n\
             s7:  {:016X}\n\
             s8:  {:016X}\n\
             s9:  {:016X}\n\
             s10: {:016X}\n\
             s11: {:016X}\n\
             t3:  {:016X}\n\
             t4:  {:016X}\n\
             t5:  {:016X}\n\
             t6:  {:016X}",
            self.ra,
            self.sp,
            self.gp,
            self.tp,
            self.t0,
            self.t1,
            self.t2,
            self.fp,
            self.s1,
            self.a0,
            self.a1,
            self.a2,
            self.a3,
            self.a4,
            self.a5,
            self.a6,
            self.a7,
            self.s2,
            self.s3,
            self.s4,
            self.s5,
            self.s6,
            self.s7,
            self.s8,
            self.s9,
            self.s10,
            self.s11,
            self.t3,
            self.t4,
            self.t5,
            self.t6,
        );
    }
}

pub struct InterruptContext {
    pub regs: InterruptRegisters,
    pub sstatus: u64,
    pub sepc: u64,
}

impl InterruptContext {
    pub fn new(regs: InterruptRegisters, sstatus: u64, sepc: u64) -> Self {
        Self { regs, sstatus, sepc }
    }

    pub fn frame_pointer(&self) -> u64 {
        self.regs.fp
    }

    pub fn stack_pointer(&self) -> u64 {
        self.regs.sp
    }

    pub fn global_pointer(&self) -> u64 {
        self.regs.gp
    }

    pub fn thread_pointer(&self) -> u64 {
        self.regs.tp
    }

    pub fn dump(&self) {
        self.regs.dump();
        log::debug!(
            r#"CSR values:
            sstatus:   {:016X}
            sepc:      {:016X}"#,
            self.sstatus,
            self.sepc
        );
        log::debug!("scause: {:016X}", riscv_rs::csr::scause::read().bits());
    }
}