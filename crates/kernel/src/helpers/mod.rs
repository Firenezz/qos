use core::{num::NonZeroU64, ptr::NonNull};

/// Places the value at the given pointer.
///
/// Returns a mutable reference to the value.
#[inline]
pub unsafe fn place_at<'a, T>(ptr: NonNull<T>, val: T) -> &'a mut T {
    unsafe {
        ptr.write(val);
        &mut *ptr.as_ptr()
    }
}

/// Places the value at the given pointer.
///
/// Returns a shared reference to the value.
#[inline]
pub unsafe fn place_at_ref<'a, T>(ptr: NonNull<T>, val: T) -> &'a T {
    unsafe {
        ptr.write(val);
        &*ptr.as_ptr()
    }
}

/// Places the value at the given `u64` address.
///
/// Returns a mutable reference to the value.
#[inline]
pub unsafe fn place_at_u64<'a, T>(addr: NonZeroU64, val: T) -> &'a mut T {
    unsafe { place_at(NonNull::new_unchecked(addr.get() as *mut T), val) }
}

/// Places the value at the given `u64` address.
///
/// Returns a shared reference to the value.
#[inline]
pub unsafe fn place_at_u64_ref<'a, T>(addr: NonZeroU64, val: T) -> &'a T {
    unsafe { place_at_ref(NonNull::new_unchecked(addr.get() as *mut T), val) }
}

/// Interprets the memory at the pointer as a mutable reference.
#[inline]
pub unsafe fn mut_at<'a, T>(ptr: NonNull<T>) -> &'a mut T {
    unsafe { &mut *ptr.as_ptr() }
}

/// Interprets the memory at the pointer as a shared reference.
#[inline]
pub unsafe fn ref_at<'a, T>(ptr: NonNull<T>) -> &'a T {
    unsafe { &*ptr.as_ptr() }
}

/// Interprets the memory at the given `u64` address as a mutable reference.
#[inline]
pub unsafe fn mut_at_u64<'a, T>(addr: NonZeroU64) -> &'a mut T {
    unsafe { mut_at(NonNull::new_unchecked(addr.get() as *mut T)) }
}

/// Interprets the memory at the given `u64` address as a shared reference.
#[inline]
pub unsafe fn ref_at_u64<'a, T>(addr: NonZeroU64) -> &'a T {
    unsafe { ref_at(NonNull::new_unchecked(addr.get() as *mut T)) }
}

/// Copies the bytes from the source to the destination.
///
/// source and destination must be aligned and non-null
pub unsafe fn copy_bytes(dst: &mut [u8], src: &[u8]) {
    unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len()) }
}

/// Zeroes the memory at the given pointer.
///
/// memory must be aligned and non-null
pub unsafe fn zero_bytes(dst: &mut [u8]) {
    unsafe { core::ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len()) }
}
