# No-crt example
This example shows how to build dstd without MSVCRT or UCRT dependencies

By avoiding UCRT, you can make the binary runnable on older Windows versions, and binary size will be slightly smaller

### Steps
1) Make `.cargo/config.toml` with the following contents:
```
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-nostdlib"]
```
(Make sure to replace the target name with a correct one that you are using)

2) Add the `no-crt` feature to Cargo.toml:
```
dstd = { version = "0.1.0", path = "..", features = ["no-crt"] }
```

3) Copy [rlibc.rs](./src/rlibc.rs) to your project and include that module:
```
#[cfg(windows)]
mod rlibc;
```
Since it redefines libc symbols, better put it behind a feature flag

After these steps, project now should compile fine without linking to UCRT

At this moment, dstd still uses some crt functions (`_localtime64_s`, `_gmtime64_s`, `_wgetenv`), but they won't be pulled in if you don't use `FormatTime` and environment variables

### Troubleshooting
1) If rustc crashes with SIGSEGV, try disabling LTO, or set it to `lto = "thin"`
