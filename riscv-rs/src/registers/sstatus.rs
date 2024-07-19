use crate::*;

use super::Action;

#[derive(Debug, Clone, Copy)]
pub struct SSTATUS {
    value: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SPP {
    User = 0b0,
    Supervisor = 0b1,
}

bitflags::bitflags! {
    pub struct SSTATUSField : usize {
        // bit 0: WPRI (write preserved and read ignored)

        /// Supervisor Interrupt Enable
        ///
        /// Is used to control whether interrupts are enabled when executing in S-mode.
        ///
        /// When SIE is set, interrupts are enabled in supervisor mode.
        /// When SIE is clear, interrupts are disabled in supervisor mode.
        const SIE = 0b1 << 1;

        // bit 2-4: WPRI

        /// Supervisor Previous Interrupt Enable
        ///
        /// Is used to save the previous interrupt enable status when an interrupt is taken while executing in S-mode.
        ///
        /// When an interrupt is taken, the current value of SIE is saved in SPIE, and SIE is set to 0.
        /// When returning from the interrupt, the value of SPIE is restored to SIE.
        const SPIE = 0b1 << 5;

        /// User Breakpoint Enable
        ///
        /// Is used to control whether user-level breakpoints are enabled when executing in S-mode.
        ///
        /// When UBE is set, user-level breakpoints are enabled in supervisor mode.
        /// When UBE is clear, user-level breakpoints are disabled in supervisor mode
        const UBE = 0b1 << 6; // The UBE field in sstatus

        const SPP = 0b1 << 8; // The SPP field in sstatus

        const SD = 0b1 << 63; // The SD field in sstatus
    }
}

impl SSTATUS {
    read_csr_as!(SSTATUS);
    write_csr!(SSTATUS);
    set_clear!(SSTATUS);
    set_clear_csr_usize!(SSTATUS, set_flags, clear_flags, SSTATUSField);

    set_clear_csr!(SSTATUS, set_sie, clear_sie, 1 << 1);

    #[inline]
    pub fn sie(&self) -> bool {
        self.value & (1 << 1) != 0
    }

    #[inline]
    pub fn spie(&self) -> bool {
        self.value & (1 << 5) != 0
    }

    #[inline]
    pub fn ube(&self) -> bool {
        self.value & (1 << 6) != 0
    }

    #[inline]
    pub fn spp(&self) -> SPP {
        match self.value & (0b1 << 8) {
            0b0 => SPP::User,
            0b1 => SPP::Supervisor,
            _ => unreachable!(),
        }
    }
}

/*bitflags! {
    impl SSTATUS: usize {
        // bit 0: WPRI (write preserved and read ignored)

        /// Supervisor Interrupt Enable
        ///
        /// Is used to control whether interrupts are enabled when executing in S-mode.
        ///
        /// When SIE is set, interrupts are enabled in supervisor mode.
        /// When SIE is clear, interrupts are disabled in supervisor mode.
        const SIE = 0b1 << 1;

        // bit 2-4: WPRI

        /// Supervisor Previous Interrupt Enable
        ///
        /// Is used to save the previous interrupt enable status when an interrupt is taken while executing in S-mode.
        ///
        /// When an interrupt is taken, the current value of SIE is saved in SPIE, and SIE is set to 0.
        /// When returning from the interrupt, the value of SPIE is restored to SIE.
        const SPIE = 0b1 << 5;

        /// User Breakpoint Enable
        ///
        /// Is used to control whether user-level breakpoints are enabled when executing in S-mode.
        ///
        /// When UBE is set, user-level breakpoints are enabled in supervisor mode.
        /// When UBE is clear, user-level breakpoints are disabled in supervisor mode
        const UBE = 0b1 << 6; // The UBE field in sstatus

        const SPP = 0b11 << 11; // The SPP field in sstatus

        const SD = 0b1 << 63; // The SD field in sstatus

        use core::arch::asm;

        #[inline]
        pub fn read() -> usize  {
            // TODO: use a macro to generate this
            let value: usize;
            unsafe {
                asm!("csrrs {}, sstatus, zero", out(reg) self.0, options(nomem, nostack, preserves_flags));
            }
            value
        }

        #[inline]
        pub fn write(value: usize) {
            unsafe {
                asm!("csrw sstatus, {}", in(reg) value, options(nomem, nostack, preserves_flags));
            }
        }



    }
}*/
