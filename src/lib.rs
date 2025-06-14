//! # BitPtr
//! [`core::ptr`] for bit offsets.
//!
//! [pointers](primitive@pointer) in Rust are locked to byte-offsets, or every 8 bits.
//! `bitptr` aims to provide methods to help read and write data at off-byte offsets.
//!
//! The primary use for `bitptr` is to pack data when the size of data is not known at compile time.
//! For types and sizes known at compile time, consider [`bilge`](https://docs.rs/bilge).


#![deny(missing_docs)]
#![no_std]


mod ptr;
pub use ptr::{ BitPtr, BitPtrMut, SubByte };


mod copy;
pub use copy::copy_nonoverlapping;

mod null;
pub use null::{ null, null_mut };
