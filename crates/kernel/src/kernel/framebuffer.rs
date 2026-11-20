//! Minimal early framebuffer drawing (Limine / ramfb smoke tests).

use crate::boot::FramebufferInfo;

/// Pack an 8-bit RGB triple into a framebuffer pixel using Limine mask shifts.
#[inline]
pub fn pack_rgb(fb: &FramebufferInfo, r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << fb.red_mask_shift)
        | ((g as u32) << fb.green_mask_shift)
        | ((b as u32) << fb.blue_mask_shift)
}

/// Fill the entire framebuffer with a solid RGB color.
///
/// Supports 24- and 32-bpp RGB framebuffers. Other depths are ignored.
///
/// # Safety
/// `fb.addr` must be a valid, writable mapping of `height * pitch` bytes for
/// the lifetime of this call (Limine guarantees that until reclaim).
pub unsafe fn clear(fb: &FramebufferInfo, r: u8, g: u8, b: u8) {
    let bytes_pp = (fb.bpp as usize) / 8;
    if bytes_pp == 0 || fb.bpp % 8 != 0 || bytes_pp > 4 {
        return;
    }

    let pixel = pack_rgb(fb, r, g, b);
    let mut pixel_bytes = [0u8; 4];
    pixel_bytes[..bytes_pp].copy_from_slice(&pixel.to_le_bytes()[..bytes_pp]);

    let base = fb.addr as *mut u8;
    let width = fb.width as usize;
    let height = fb.height as usize;
    let pitch = fb.pitch as usize;

    for y in 0..height {
        let row = unsafe { base.add(y * pitch) };
        for x in 0..width {
            let dst = unsafe { row.add(x * bytes_pp) };
            for (i, byte) in pixel_bytes[..bytes_pp].iter().enumerate() {
                unsafe { core::ptr::write_volatile(dst.add(i), *byte) };
            }
        }
    }
}
