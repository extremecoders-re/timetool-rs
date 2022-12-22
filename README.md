# timetool-rs

Synchronize Windows Time using NTP. Useful when UDP source port 123 is blocked but NTP traffic is itself allowed on other ports.

## Compilation

Compiles directly on windows.

## Cross-compilation

Follow the steps as shown in https://bevy-cheatbook.github.io/setup/cross/linux-windows.html

Cross-compilation with both msvc & gnu toolchain are supported.

```
cargo build --target x86_64-pc-windows-msvc --release

cargo build --target x86_64-pc-windows-gnu --release
```

To link statically, add the following to rustflags in `.cargo/config.toml`

```
rustflags = ["-C", "target-feature=+crt-static"]
```
From https://github.com/KodrAus/rust-cross-compile/blob/main/README.md