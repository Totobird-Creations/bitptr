use core::cmp::{ PartialOrd, Ord, Ordering };
use core::mem;


mod subbyte;
pub use subbyte::SubByte;


macro_rules! bitptr { (
    $( #[doc = $doc:tt] )*
    $ident:ident, $byte:ty
) => {

    $( #[doc = $doc] )*
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct $ident {
        byte : $byte,
        bit  : SubByte
    }

    impl PartialOrd for $ident {
        #[inline(always)]
        fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
            Some(Ord::cmp(self, other))
        }
    }

    impl Ord for $ident {
        fn cmp(&self, other : &Self) -> Ordering {
            self.byte.cmp(&other.byte)
                .then_with(|| self.bit.cmp(&other.bit))
        }
    }

    impl $ident {

        /// Create a new raw bit pointer from a raw byte pointer and a sub-byte bit offset.
        #[inline(always)]
        pub const fn new(byte : $byte, subbyte_bit : SubByte) -> Self {
            Self { byte, bit : subbyte_bit }
        }

        /// Create a new raw bit pointer from a raw byte pointer and zero bit offset.
        #[inline(always)]
        pub const fn new_on_byte(byte : $byte) -> Self {
            Self { byte, bit : SubByte::MIN }
        }

        /// Create a new raw bit pointer from a raw byte pointer and a bit offset.
        ///
        /// # Safety
        /// `new_with_offset` has the same safety concerns as [`(*const _)::offset`](primitive@pointer#method.byte_offset).
        #[inline(always)]
        pub const unsafe fn new_with_offset(byte : $byte, bit_count : isize) -> Self {
            unsafe { Self::new_on_byte(byte).bit_offset(bit_count) }
        }

    }

    impl $ident {

        /// Returns the raw byte pointer, rounded down.
        #[inline]
        pub const fn floor_byte(&self) -> $byte { self.byte }

        /// Returns the sub-byte bit offset of this raw bit pointer.
        #[inline]
        pub const fn subbyte_bit(&self) -> SubByte { self.bit }

        /// Returns the raw byte pointer (rounded down) and sub-byte bit offset of this raw bit pointer.
        #[inline]
        pub const fn as_inner(&self) -> ($byte, SubByte,) { (self.byte, self.bit,) }

    }

    impl $ident {

        #[allow(clippy::missing_safety_doc)]
        /// Adds a signed offset in bytes to a bit pointer.
        ///
        /// `count` is in a unit of **bytes**.
        ///
        /// Analagous to [`(*const _)::byte_offset`](primitive@pointer#method.byte_offset).
        #[inline]
        pub const unsafe fn byte_offset(mut self, count : isize) -> Self {
            self.byte = unsafe { self.byte.byte_offset(count) };
            self
        }

        #[allow(clippy::missing_safety_doc)]
        /// Adds a signed offset in bits to a bit pointer.
        ///
        /// `count` is in a unit of **bits**.
        ///
        /// Analagous to [`(*const _)::byte_offset`](primitive@pointer#method.byte_offset).
        #[inline]
        pub const unsafe fn bit_offset(mut self, count : isize) -> Self {
            let bit         = (self.bit.get() as isize) + count;
            let byte_offset = bit.div_euclid(8);
            let bit         = bit.rem_euclid(8) as u8;
            self.byte       = unsafe { self.byte.byte_offset(byte_offset) };
            self.bit        = unsafe { SubByte::new_unchecked(bit) };
            self
        }

        /// Adds a signed offset in bytes to a bit pointer using wrapping arithmetic.
        ///
        /// `count` is in a unit of **bytes**.
        ///
        /// Analagous to [`(*const _)::wrapping_byte_offset`](primitive@pointer#method.wrapping_byte_offset).
        #[inline]
        pub const fn wrapping_byte_offset(mut self, count : isize) -> Self {
            self.byte = self.byte.wrapping_byte_offset(count);
            self
        }

    }

    impl $ident {

        /// Reads the bit that is pointed to.
        ///
        /// # Safety
        /// Behaviour is undefined if `self.floor_byte()` is not [valid](core::ptr#safety) for reads.
        pub const unsafe fn read(self) -> bool {
            (((unsafe { *self.byte }) << self.bit.get()) & 0b10000000) != 0
        }

    }


} }


bitptr! {
    /// A pointer to a bit in memory.
    ///
    /// Analagous to [`*const T`](core::ptr).
    BitPtr, *const u8
}

impl BitPtr {

    /// Convert to a [`BitPtrMut`] with the same byte and bit offset.
    #[inline(always)]
    pub fn as_mut(self) -> BitPtrMut {
        unsafe { mem::transmute(self) }
    }

}


bitptr! {
    /// A mutable pointer to a bit in memory.
    ///
    /// Analagous to [`*mut T`](core::ptr).
    BitPtrMut, *mut u8
}

impl BitPtrMut {

    /// Convert to a [`BitPtr`] with the same byte and bit offset.
    #[inline(always)]
    pub fn as_const(self) -> BitPtr {
        unsafe { mem::transmute(self) }
    }

    /// Sets the bit that is pointed to.
    ///
    /// # Safety
    /// Behaviour is undefined if `self.floor_byte()` is not [valid](core::ptr#safety) for writes.
    pub const unsafe fn write(self, bit : bool) {
        let mask = ((u8::MAX << self.bit.get()) & 0b10000000) >> self.bit.get();
        if (bit) {
            unsafe { *self.byte |= mask; }
        } else {
            unsafe { *self.byte &= ! mask; }
        }
    }


}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitptr_new_offset() {
        let     x = 0b0101101110010110u16.to_be();
        let mut y = 0b1111111111111111u16.to_be();

        let xptr = unsafe { BitPtr::new_with_offset(&x as *const _ as *const _, 7) };
        assert_eq!(xptr.floor_byte(), &x as *const _ as *const _);
        assert_eq!(xptr.subbyte_bit().get(), 7);

        let yptr = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 3) };
        assert_eq!(yptr.floor_byte(), &mut y as *mut _ as *mut _);
        assert_eq!(yptr.subbyte_bit().get(), 3);

        let yptr1 = unsafe { BitPtrMut::new_with_offset(&mut y as *mut _ as *mut _, 13) };
        assert_eq!(yptr1.floor_byte(), unsafe { (&mut y as *mut _ as *mut u8).byte_add(1) });
        assert_eq!(yptr1.subbyte_bit().get(), 5);

    }

    #[test]
    fn bitptr_read() {
        let x = 0b01001110u8.to_be();

        let mut xptr = BitPtr::new_on_byte(&x as *const _ as *const _);
        assert_eq!(unsafe { xptr.read() }, false);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, true);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, false);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, false);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, true);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, true);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, true);

        xptr = unsafe { xptr.bit_offset(1) };
        assert_eq!(unsafe { xptr.read() }, false);

    }

    #[test]
    fn bitptr_write() {
        let mut x = 0b01001110u8.to_be();

        let mut xptr = BitPtrMut::new_on_byte(&mut x as *mut _ as *mut _);
        unsafe { xptr.write(true); }
        assert_eq!(u8::from_be(x), 0b11001110u8);

        xptr = unsafe { xptr.bit_offset(5) };
        unsafe { xptr.write(false); }
        assert_eq!(u8::from_be(x), 0b11001010u8);

    }

}
