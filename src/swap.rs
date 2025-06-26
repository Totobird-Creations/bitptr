use crate::BitPtrMut;
use core::ptr;


/// Swaps `bit_count` bits between the two regions of memory beginning at `x` and `y`. The two regions must *not* overlap.
///
/// `bitptr` does not provide a `swap` function for memory which might overlap.
///
/// The copy is "untyped" in the sense that data may be uninitialized. The initialization state is preserved exactly.
///
///
/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
/// - `x.floor_byte()` must be [valid](core::ptr#safety) for reads of `((x.subbyte_bit().get() as usize) + bit_count).div_ceil(8)` bytes.
/// - `y.floor_byte()` must be [valid](core::ptr#safety) for writes of `((y.subbyte_bit().get() as usize) + bit_count).div_ceil(8)` bytes.
/// - The region of memory beginning at `x` with a size of `bit_count` bits must *not* overlap with the region of memory beginning at `y` with the same size.
///   The byte region may overlap. The relevant bits themselves may not.
///
///
/// ---
/// Analagous to [`ptr::swap_nonoverlapping`](core::ptr::swap_nonoverlapping).
pub unsafe fn swap_nonoverlapping(x : BitPtrMut, y : BitPtrMut, bit_count : usize) {
    if (bit_count == 0) { return; }

    let (x_byte, x_bit,) = x.as_inner();
    let x_bit_l = x_bit.get() as usize;
    let x_bit_r = (8isize - ((x_bit_l + bit_count) as isize)).rem_euclid(8);
    let (y_byte, y_bit,) = y.as_inner();
    let y_bit_l = y_bit.get() as usize;
    let y_bit_r = (8isize - ((y_bit_l + bit_count) as isize)).rem_euclid(8);

    let mut x_rolling = u32::from_be(unsafe { ptr::read(x_byte.byte_sub(2) as *const u32) });
    let mut y_rolling = u32::from_be(unsafe { ptr::read(y_byte.byte_sub(2) as *const u32) });;

    let x_byte_count = (x_bit_l + bit_count).div_ceil(8);
    let y_byte_count = (y_bit_l + bit_count).div_ceil(8);
    for offset in 0..(x_byte_count.max(y_byte_count)) {
        let x_byte = unsafe { x_byte.byte_add(offset) };
        let y_byte = unsafe { y_byte.byte_add(offset) };

        // Get masks over the bits to write.
        let mut x_mask = u8::MAX;
        let mut y_mask = u8::MAX;
        if (offset == 0) {
            x_mask = x_mask << x_bit_l >> x_bit_l;
            y_mask = y_mask << y_bit_l >> y_bit_l;
        }
        match ((offset + 2).saturating_sub(x_byte_count)) {
            0 => { }
            1 => { x_mask = x_mask >> x_bit_r << x_bit_r; },
            _ => { x_mask = 0b00000000; }
        }
        match ((offset + 2).saturating_sub(y_byte_count)) {
            0 => { }
            1 => { y_mask = y_mask >> y_bit_r << y_bit_r; },
            _ => { y_mask = 0b00000000; }
        }

        // Build teh bytes that will be written.
        let x_src_b = ((y_rolling << y_bit_l >> (8 + x_bit_l)) & 0b11111111) as u8;
        let y_src_b = ((x_rolling << x_bit_l >> (8 + y_bit_l)) & 0b11111111) as u8;

        // Get the bytes to edit.
        let mut x_dst_b = unsafe { *x_byte };
        let mut y_dst_b = unsafe { *y_byte };

        // Wipe the bits that will be overwritten.
        x_dst_b &= ! x_mask;
        y_dst_b &= ! y_mask;

        // Write the relevant bits.
        x_dst_b |= x_src_b & x_mask;
        y_dst_b |= y_src_b & y_mask;

        // Overwrite the bytes.
        unsafe { *x_byte = x_dst_b; }
        unsafe { *y_byte = y_dst_b; }

        // Roll the rolling values.
        x_rolling = (x_rolling << 8) | ((unsafe { x_byte.byte_add(1).read() }) as u32);
        y_rolling = (y_rolling << 8) | ((unsafe { y_byte.byte_add(1).read() }) as u32);

    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn swap_within_byte_boundary() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b1110100011010010u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 9) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Swap and check final values.
        unsafe { swap_nonoverlapping(xptr, yptr, 4); }
        assert_eq!(u16::from_be(x), 0b0101101110100110u16);
        assert_eq!(u16::from_be(y), 0b1110010011010010u16);
    }


    #[test]
    fn swap_across_byte_boundary() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b1110100011010010u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 7) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Swap and check final values.
        unsafe { swap_nonoverlapping(xptr, yptr, 7); }
        assert_eq!(u16::from_be(x), 0b0101101010001110u16);
        assert_eq!(u16::from_be(y), 0b1111100101010010u16);
    }


    #[test]
    fn swap_aligned_start() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b1110100011010010u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 8) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 0) };

        // Swap and check final values.
        unsafe { swap_nonoverlapping(xptr, yptr, 5); }
        assert_eq!(u16::from_be(x), 0b0101101111101110u16);
        assert_eq!(u16::from_be(y), 0b1001000011010010u16);
    }


    #[test]
    fn swap_aligned_end() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b1110100011010010u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 5) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 13) };

        // Swap and check final values.
        unsafe { swap_nonoverlapping(xptr, yptr, 3); }
        assert_eq!(u16::from_be(x), 0b0101101010010110u16);
        assert_eq!(u16::from_be(y), 0b1110100011010011u16);
    }


    #[test]
    fn swap_different_byte_width() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b1110100011010010u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 2) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 6) };

        // Swap and check final values.
        unsafe { swap_nonoverlapping(xptr, yptr, 5); }
        assert_eq!(u16::from_be(x), 0b0100110110010110u16);
        assert_eq!(u16::from_be(y), 0b1110100110110010u16);
    }


}
