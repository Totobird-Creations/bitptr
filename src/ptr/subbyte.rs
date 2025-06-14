/// A sub-byte offset.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubByte {
    bit : u8
}

impl SubByte {

    /// The minimum sub-byte offset value (`0`).
    pub const MIN : Self = unsafe { Self::new_unchecked(0) };

    /// The maximum sub-byte offset value (`7`).
    pub const MAX : Self = unsafe { Self::new_unchecked(7) };

    /// Create a new `SubByte` from a sub-byte offset.
    ///
    /// # Returns
    /// Returns `None` if `bit` is `8` or greater, as it is not valid.
    #[inline]
    pub const fn new(bit : u8) -> Option<Self> {
        if (bit >= 8) { None }
        else { Some(Self { bit }) }
    }

    /// Create a new `SubByte` from a sub-byte offset, without checking if it is valid.
    ///
    /// # Safety
    /// Behaviour is undefined if `bit` is not **less** than `8`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(bit : u8) -> Self { Self { bit } }

    /// Returns the sub-byte offset as a `u8`.
    #[inline]
    pub const fn get(&self) -> u8 { self.bit }

}
