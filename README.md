# dstd: Dragon's standard library
Lightweight and feature-complete std replacement

New types that do not exist in std:
- `dstd::ffi::OsStr` represents a null-terminated C string in platform encoding, making it much easier to work with windows functions. Also, it is serializable, and is not opaque
- `dstd::time::FormatTime` represents a broken-down representation of `SystemTime`, suitable for human-readable output (`struct tm`/`SYSTEMTIME` in C)

Advantages over std:
- Much smaller binary sizes! Only 4872 bytes for helloworld, almost matches C (`aarch64-linux-android`, full lto, strip = true). Recommended for cdylib crates
- Not prebuilt, so supports lto without nightly. Release builds in 1 second on a phone

Disadvantages:
- ALPHA QUALITY SOFTWATE. May contain terrible bugs which otherwise wouldn't happen on std
- HIGH AMOUNT OF UNSAFE CODE. WAS NOT AUDITED! May contain multiple undiscovered security vulnerabilities, do not use in production!
- Only supports Windows, Linux and Android! Doesn't even support OSX (and I don't have a Mac)
- Almost entirely incompatible to std, not a drop-in replacement. However overall crate structure and type/function names should be familiar

Also, to take advantage of dstd fully, all crates in your dependency tree should be `no_std` or use dstd. This is not the case for usual Rust ecosystem crates, so it might be useless for your Rust program

This crate was tested on:
- `aarch64-linux-android` Android 16
- `aarch64-unknown-linux-gnu` Aarch64 Linux 4.19 with glibc (same phone)
- `x86_64-pc-windows-gnullvm` x86_64 Wine 10.10 (again, same phone)

Proudly written in nano™
