//! NS16550-compatible UART (QEMU `virt` `serial@10000000`, etc.).

use core::fmt::{self, Write};
use core::num::NonZeroU64;

/// Register offsets (byte-spaced; QEMU `ns16550a` uses `reg-shift = 0`).
const RBR: u8 = 0; // DLAB=0 read
const THR: u8 = 0; // DLAB=0 write
const IER: u8 = 1;
const FCR: u8 = 2;
const LCR: u8 = 3;
const MCR: u8 = 4;
const LSR: u8 = 5;

const LCR_WLEN8: u8 = 0x03; // 8N1
const FCR_ENABLE: u8 = 0x01;
const FCR_CLEAR: u8 = 0x06;
const MCR_DTR_RTS: u8 = 0x03;
const LSR_THRE: u8 = 1 << 5;
const LSR_DR: u8 = 1 << 0;

/// Platform UART backend. Extend when a non-16550 console appears.
pub enum Uart {
    NS16550(NS16550),
}

impl Uart {
    pub fn new_ns16550(base: NonZeroU64) -> Self {
        Self::NS16550(NS16550::new(base))
    }

    /// Disable IRQs, enable FIFO, 8N1. Leaves baud divisor alone (QEMU preset).
    pub fn init(&self) {
        match self {
            Self::NS16550(u) => u.init(),
        }
    }

    pub fn put_byte(&self, byte: u8) {
        match self {
            Self::NS16550(u) => u.put_byte(byte),
        }
    }

    pub fn write_str_raw(&self, s: &str) {
        match self {
            Self::NS16550(u) => u.write_str_raw(s),
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_raw(s);
        Ok(())
    }
}

pub struct NS16550 {
    base: NonZeroU64,
}

impl NS16550 {
    pub const fn new(base: NonZeroU64) -> Self {
        Self { base }
    }

    pub fn init(&self) {
        // SAFETY: `base` is a mapped NS16550 MMIO window.
        unsafe {
            self.write_reg(IER, 0);
            self.write_reg(FCR, FCR_ENABLE | FCR_CLEAR);
            self.write_reg(LCR, LCR_WLEN8);
            self.write_reg(MCR, MCR_DTR_RTS);
        }
    }

    pub fn put_byte(&self, byte: u8) {
        if byte == b'\n' {
            self.put_byte_raw(b'\r');
        }
        self.put_byte_raw(byte);
    }

    pub fn write_str_raw(&self, s: &str) {
        for byte in s.bytes() {
            self.put_byte(byte);
        }
    }

    fn put_byte_raw(&self, byte: u8) {
        while self.read_reg(LSR) & LSR_THRE == 0 {
            core::hint::spin_loop();
        }
        // SAFETY: THR is a writable data register in the mapped window.
        unsafe {
            self.write_reg(THR, byte);
        }
    }

    #[allow(dead_code)]
    pub fn try_get_byte(&self) -> Option<u8> {
        if self.read_reg(LSR) & LSR_DR == 0 {
            return None;
        }
        Some(self.read_reg(RBR))
    }

    #[inline]
    fn reg_ptr(&self, offset: u8) -> *mut u8 {
        (self.base.get() as *mut u8).wrapping_add(offset as usize)
    }

    #[inline]
    unsafe fn write_reg(&self, offset: u8, value: u8) {
        unsafe {
            core::ptr::write_volatile(self.reg_ptr(offset), value);
        }
    }

    #[inline]
    fn read_reg(&self, offset: u8) -> u8 {
        // SAFETY: offset is a defined NS16550 register in a mapped window.
        unsafe { core::ptr::read_volatile(self.reg_ptr(offset)) }
    }
}

impl Write for NS16550 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_raw(s);
        Ok(())
    }
}
