# BitPtr
[`core::ptr`](https://doc.rust-lang.org/stable/core/ptr/index.html) for bit offsets.

[pointers](https://doc.rust-lang.org/stable/core/primitive.pointer.html) in Rust are locked to byte-offsets, or every 8 bits.
`bitptr` aims to provide methods to help read and write data at off-byte offsets.

The primary use for `bitptr` is to pack data when the size of data is not known at compile time.
For types and sizes known at compile time, consider [`bilge`](https://docs.rs/bilge).
