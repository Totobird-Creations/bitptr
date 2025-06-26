use crate::BitPtrMut;


/// Fills `bit_count` bits at `dst`.
///
/// If `value` is `true`, the bit range will be filled with `1`. `0` for `false`.
///
/// # Safety
/// Behaviour is undefined if `src.floor_byte()` is not [valid](core::ptr#safety) for writes of `((dst.subbyte_bit().get() as usize) + bit_count).div_ceil(8)` bytes.
pub unsafe fn fill(dst : BitPtrMut, bit_count : usize, value : bool) {
    if (bit_count == 0) { return; }

    let (dst_byte, dst_bit,) = dst.as_inner();
    let dst_bit_l = dst_bit.get() as usize;
    let dst_bit_r = (8isize - ((dst_bit_l + bit_count) as isize)).rem_euclid(8);

    let dst_byte_count = (dst_bit_l + bit_count).div_ceil(8);
    for dst_offset in 0..dst_byte_count {
        let dst_byte = unsafe { dst_byte.byte_add(dst_offset) };

        // Get a mask over the bits to write.
        let mut mask = u8::MAX;
        if (dst_offset == 0) {
            mask = mask << dst_bit_l >> dst_bit_l;
        }
        if (dst_offset + 1 == dst_byte_count) {
            mask = mask >> dst_bit_r << dst_bit_r;
        }

        // Fill the relevant bit range.
        if (value) {
            unsafe { *dst_byte |= mask; }
        } else {
            unsafe { *dst_byte &= ! mask; }
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn fill_within_byte_boundary() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b0101101110010110u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 9) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Fill and check final value.
        unsafe { fill(xptr, 4, false); }
        unsafe { fill(yptr, 4, true); }
        let x = u16::from_be(x);
        let y = u16::from_be(y);
        assert_eq!(x, 0b0101101110000110u16);
        assert_eq!(y, 0b0101111110010110u16);
    }


    #[test]
    fn fill_across_byte_boundary() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b0101101110010110u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 7) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };

        // Fill and check final value.
        unsafe { fill(xptr, 7, false); }
        unsafe { fill(yptr, 7, true); }
        let x = u16::from_be(x);
        let y = u16::from_be(y);
        assert_eq!(x, 0b0101101000000010u16);
        assert_eq!(y, 0b0101111111010110u16);
    }


    #[test]
    fn fill_aligned_start() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b0101101110010110u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 8) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 0) };

        // Fill and check final value.
        unsafe { fill(xptr, 5, false); }
        unsafe { fill(yptr, 5, true); }
        let x = u16::from_be(x);
        let y = u16::from_be(y);
        assert_eq!(x, 0b0101101100000110u16);
        assert_eq!(y, 0b1111101110010110u16);
    }


    #[test]
    fn fill_aligned_end() {
        let mut x = 0b0101101110010110u16.to_be();
        let mut y = 0b0101101110010110u16.to_be();

        let xptr = unsafe { BitPtrMut::new_with_offset(&mut x as *mut _ as *mut _, 5) };
        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 13) };

        // Fill and check final value.
        unsafe { fill(xptr, 3, false); }
        unsafe { fill(yptr, 3, true); }
        let x = u16::from_be(x);
        let y = u16::from_be(y);
        assert_eq!(x, 0b0101100010010110u16);
        assert_eq!(y, 0b0101101110010111u16);
    }

}
