//! Trap cause codes (`scause`).

/// Asynchronous interrupt codes (`scause.interrupt == 1`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum Interrupt {
    SupervisorSoftware = 1,
    SupervisorTimer = 5,
    SupervisorExternal = 9,
}

impl Interrupt {
    pub fn from_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::SupervisorSoftware),
            5 => Some(Self::SupervisorTimer),
            9 => Some(Self::SupervisorExternal),
            _ => None,
        }
    }
}

/// Synchronous exception codes (`scause.interrupt == 0`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum Exception {
    InstructionMisaligned = 0,
    InstructionFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadMisaligned = 4,
    LoadFault = 5,
    StoreMisaligned = 6,
    StoreFault = 7,
    UserEnvCall = 8,
    SupervisorEnvCall = 9,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
}

impl Exception {
    pub fn from_code(code: u64) -> Option<Self> {
        match code {
            0 => Some(Self::InstructionMisaligned),
            1 => Some(Self::InstructionFault),
            2 => Some(Self::IllegalInstruction),
            3 => Some(Self::Breakpoint),
            4 => Some(Self::LoadMisaligned),
            5 => Some(Self::LoadFault),
            6 => Some(Self::StoreMisaligned),
            7 => Some(Self::StoreFault),
            8 => Some(Self::UserEnvCall),
            9 => Some(Self::SupervisorEnvCall),
            12 => Some(Self::InstructionPageFault),
            13 => Some(Self::LoadPageFault),
            15 => Some(Self::StorePageFault),
            _ => None,
        }
    }
}
