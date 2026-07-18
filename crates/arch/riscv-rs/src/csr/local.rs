//! Local shadow copy of a CSR for multi-field read-modify-write transactions.

use super::{Csr, Readable, Writeable};

/// Shadow value of a CSR held in a general-purpose register / stack slot.
///
/// Mutate typed fields, then [`commit`](Self::commit) with a single `csrw`, or
/// [`commit_set_clear`](Self::commit_set_clear) to emit `csrrs`/`csrrc` for only
/// the bits that changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Local<R: Csr> {
    bits: R::Bits,
    _marker: core::marker::PhantomData<R>,
}

impl<R: Csr> Local<R> {
    /// Construct a local view from raw bits (no CSR access).
    #[inline(always)]
    pub const fn from_bits(bits: R::Bits) -> Self {
        Self {
            bits,
            _marker: core::marker::PhantomData,
        }
    }

    /// Raw bit pattern.
    #[inline(always)]
    pub const fn bits(self) -> R::Bits {
        self.bits
    }

    /// Mutable access to the raw bit pattern (for CSR-specific helpers).
    #[inline(always)]
    pub fn bits_mut(&mut self) -> &mut R::Bits {
        &mut self.bits
    }
}

impl<R: Readable> Local<R> {
    /// Read the CSR into a local shadow (`csrr`).
    #[inline(always)]
    pub fn read() -> Self {
        Self::from_bits(R::read_raw())
    }
}

impl<R: Writeable> Local<R> {
    /// Write the full local value to the CSR (`csrw`).
    #[inline(always)]
    pub fn commit(self) {
        R::write_raw(self.bits);
    }
}

impl<R: Writeable<Bits = u64>> Local<R> {
    /// Commit only bits that differ from `original` via `csrrs` / `csrrc`.
    ///
    /// Useful when a full `csrw` would clobber W1C or other sticky fields.
    #[inline(always)]
    pub fn commit_set_clear(self, original: Self) {
        let set_mask = self.bits & !original.bits;
        let clear_mask = original.bits & !self.bits;
        if set_mask != 0 {
            R::set_bits(set_mask);
        }
        if clear_mask != 0 {
            R::clear_bits(clear_mask);
        }
    }
}
