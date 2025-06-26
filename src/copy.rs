use crate::{ BitPtr, BitPtrMut };


/// Copies `bit_count` bits from `src` to `dst`. The source and destination must *not* overlap.
///
/// `bitptr` does not provide a `copy` function for memory which might overlap.
///
/// The copy is "untyped" in the sense that data may be uninitialized. The initialization state is preserved exactly.
///
///
/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
/// - `src.floor_byte()` must be [valid](core::ptr#safety) for reads of `((src.subbyte_bit().get() as usize) + bit_count).div_ceil(8)` bytes.
/// - `dst.floor_byte()` must be [valid](core::ptr#safety) for writes of `((dst.subbyte_bit().get() as usize) + bit_count).div_ceil(8)` bytes.
/// - The region of memory beginning at `src` with a size of `bit_count` bits must *not* overlap with the region of memory beginning at `dst` with the same size.
///   The byte region may overlap. The relevant bits themselves may not.
///
///
/// # Footguns
///
/// Make sure to account for endianness.
/// ```rust should_panic
/// use bitptr::{ BitPtr, BitPtrMut };
///
/// let     x = 0b_0101101110010110_u16;
/// //                    ^^^^^^^ This is the region that is 'supposed to' be read from.
/// let mut y = 0b_1111111111111111_u16;
/// //                ^^^^^^^ This is the region that is 'supposed to' be written to.
///
/// let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 7) };
/// let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };
///
/// unsafe { bitptr::copy_nonoverlapping(xptr, yptr, 7); }
/// assert_eq!(y, 0b_1111100101111111_u16);
/// ```
/// On a little-endian system, the `assert_eq!` in the code above will panic. The final value of `y` is `0b_1011111111100101_u16`, *not* `0b_1111100101111111_u16`.
///
/// Explicitely converting from native-endian to big-endian before, and big-endian to native-endian after, can solve this issue.
/// ```rust
/// use bitptr::{ BitPtr, BitPtrMut };
///
/// let     x = 0b_0101101110010110_u16.to_be();
/// //                    ^^^^^^^ This is the region that is 'supposed to' be read from.
/// let mut y = 0b_1111111111111111_u16.to_be();
/// //                ^^^^^^^ This is the region that is 'supposed to' be written to.
///
/// let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 7) };
/// let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };
///
/// unsafe { bitptr::copy_nonoverlapping(xptr, yptr, 7); }
/// let y = u16::from_be(y);
/// assert_eq!(y, 0b_1111100101111111_u16);
/// ```
///
///
/// ---
/// Analagous to [`ptr::copy_nonoverlapping`](core::ptr::copy_nonoverlapping).
pub unsafe fn copy_nonoverlapping(src : BitPtr, dst : BitPtrMut, bit_count : usize) {
    if (bit_count == 0) { return; }

    let (src_byte, src_bit,) = src.as_inner();
    let src_bit_l = src_bit.get() as usize;
    let (dst_byte, dst_bit,) = dst.as_inner();
    let dst_bit_l = dst_bit.get() as usize;
    let dst_bit_r = (8isize - ((dst_bit_l + bit_count) as isize)).rem_euclid(8);

    let dst_byte_count = (dst_bit_l + bit_count).div_ceil(8);
    for dst_offset in 0..dst_byte_count {
        let src_byte = unsafe { src_byte.byte_add(dst_offset) };
        let dst_byte = unsafe { dst_byte.byte_add(dst_offset) };

        // Get a mask over the bits to write.
        let mut mask = u8::MAX;
        if (dst_offset == 0) {
            mask = mask << dst_bit_l >> dst_bit_l;
        }
        if (dst_offset + 1 == dst_byte_count) {
            mask = mask >> dst_bit_r << dst_bit_r;
        }

        // Build the byte that will be written.
        let src_b = ((
            (((unsafe { *src_byte.byte_sub(1) } as u32) << 16)
            | ((unsafe { *src_byte } as u32) << 8)
            | (unsafe { *src_byte.byte_add(1) } as u32))
            << src_bit_l
            >> (8 + dst_bit_l)
        ) & 0b11111111) as u8;

        // Get the byte to edit.
        let mut dst_b = unsafe { *dst_byte };

        // Wipe the bits that will be overwritten.
        dst_b &= ! mask;

        // Write the relevant bits.
        dst_b |= src_b & mask;

        // Overwrite the byte.
        unsafe { *dst_byte = dst_b; }

    }

}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn byte_order_sanity_check() {
        let x = 0b0101101110010110u16.to_be();
        assert_eq!(unsafe { *(&x as *const _ as *const u8) }, 0b01011011);
        assert_eq!(unsafe { *(&x as *const _ as *const u8).byte_add(1) }, 0b10010110);
    }


    #[test]
    fn copy_within_byte_boundary() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b0000000000000000u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 9) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 4); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b0000010000000000u16);
    }


    #[test]
    fn copy_across_byte_boundary() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 7) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 7); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b1111100101111111u16);
    }


    #[test]
    fn copy_aligned_start() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 8) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 0) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 5); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b1001011111111111u16);
    }


    #[test]
    fn copy_aligned_end() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 5) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 13) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 3); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b1111111111111011u16);
    }


    #[test]
    fn copy_src_wider_than_dst() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 6) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 2) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 5); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b1111100111111111u16);
    }


    #[test]
    fn copy_dst_wider_than_src() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 2) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 6) };

        // Copy and check final value.
        unsafe { copy_nonoverlapping(xptr, yptr, 5); }
        let y = u16::from_be(y);
        assert_eq!(y, 0b1111110110111111u16);
    }


}
