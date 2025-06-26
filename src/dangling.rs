use crate::{ BitPtr, BitPtrMut };
use core::ptr;


/// Creates a dangling raw bit pointer.
///
/// ---
/// Analagous to [`ptr::dangling`](core::ptr::dangling).
#[inline(always)]
pub const fn dangling() -> BitPtr {
    BitPtr::new_on_byte(ptr::dangling())
}


/// Creates a dangling mutable raw bit pointer.
///
/// ---
/// Analagous to [`ptr::dangling_mut`](core::ptr::dangling_mut).
#[inline(always)]
pub const fn dangling_mut() -> BitPtrMut {
    BitPtrMut::new_on_byte(ptr::dangling_mut())
}
