use crate::{read_csr, read_csr_as, write_csr, write_csr_usize};

use super::general::fp::read;

pub struct SATP {
    value: usize,
}

impl SATP {
    read_csr_as!(SATP);
    write_csr_usize!(SATP);

    #[inline]
    pub fn as_usize(&self) -> usize {
        read();
        self.value
    }

    #[inline]
    pub fn asid(&self) -> usize {
        self.value & 0x0FFFFF
    }

    #[inline]
    pub fn mode(&self) -> usize {
        (self.value >> 60) & 0x3
    }
}
