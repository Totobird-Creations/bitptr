use crate::{ BitPtr, BitPtrMut };
use core::ptr;


/// Creates a null raw bit pointer.
///
/// ---
/// Analagous to [`ptr::null`](core::ptr::null).
#[inline(always)]
pub const fn null() -> BitPtr {
    BitPtr::new_on_byte(ptr::null())
}


/// Creates a null mutable raw bit pointer.
///
/// ---
/// Analagous to [`ptr::null_mut`](core::ptr::null_mut).
#[inline(always)]
pub const fn null_mut() -> BitPtrMut {
    BitPtrMut::new_on_byte(ptr::null_mut())
}
