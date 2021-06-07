# Build Rust Firmware

## Tools

1. Install [RUST](https://www.rust-lang.org/)

1.1. Intall xbuild

```
cargo install cargo-xbuild
```

2. Install [NASM](https://www.nasm.us/)

## Build

1. [rust-ipl](https://github.com/jyao1/rust-firmware/tree/master/rust-ipl)

Enter below in Visual Studio 2019 X64 CMD windows.
Note that [ResetVector](https://github.com/jyao1/rust-firmware/tree/master/rust-ipl/ResetVector) will also be built via build scripts.

```
cargo xbuild --target x86_64-unknown-uefi --release
```

2. [rust-uefi-payload](https://github.com/jyao1/rust-firmware/tree/master/rust-uefi-payload)

```
cargo xbuild --target target.json --release
```

3. [rust-firmware-tool](https://github.com/jyao1/rust-firmware/tree/master/rust-firmware-tool)

```
cargo build
```

Assemble all images together

```
set RESET_VECTOR_BIN=<rust-firmware>\rust-ipl\ResetVector\ResetVector.bin
set RUST_IPL_BIN=<rust-firmware>\rust-ipl\target\x86_64-unknown-uefi\release\rust_ipl.efi
set RUST_PAYLOAD_BIN=<rust-firmware>\rust-uefi-payload\target\target\release\rust-uefi-payload
set RUST_FIRMWARE_BIN=<rust-firmware>\rust-firmware-tool\final.bin
cargo run -p rust-firmware-tool -- %RESET_VECTOR_BIN% %RUST_IPL_BIN% %RUST_PAYLOAD_BIN% %RUST_FIRMWARE_BIN%
```

## Run

```
set QEMU_BIN="C:\Program Files\qemu\qemu-system-x86_64.exe"
set BIOS_BIN=<rust-firmware>\rust-firmware-tool\final.bin
set SERIAL_LOG=-serial file:serial.log
%QEMU_BIN% -machine q35 -drive if=pflash,format=raw,unit=0,file=%BIOS_BIN%,readonly=on %SERIAL_LOG%
```
