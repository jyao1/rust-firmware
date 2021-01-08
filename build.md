# Build Rust Firmware

## Tools

1. Install [RUST](https://www.rust-lang.org/)

1.1. Intall xbuild

```
cargo install cargo-xbuild
```

2. Install [NASM](https://www.nasm.us/)

## Build

1. [ResetVector](https://github.com/jyao1/rust-firmware/tree/master/rust-ipl/ResetVector)

```
nasm -f bin ResetVector.nasm -o ResetVector.bin -DARCH_X64 -DPAGE_TABLE_SIZE=0x6000 -DPAGE_TABLE_BASE=0x800000  -DSEC_TOP_OF_STACK=0x830000
```

2. [rust-ipl](https://github.com/jyao1/rust-firmware/tree/master/rust-ipl)

Enter Visual Studio 2019 X64 CMD windows

```
cargo xbuild --target x86_64-unknown-uefi
```

3. [rust-uefi-payload](https://github.com/jyao1/rust-firmware/tree/master/rust-uefi-payload)

```
cargo xbuild --target target.json
```

4. [rust-firmware-tool](https://github.com/jyao1/rust-firmware/tree/master/rust-firmware-tool)

```
cargo build
```

Assemble all images together

```
set RESET_VECTOR_BIN=<rust-firmware>\rust-ipl\ResetVector\ResetVector.bin
set RUST_IPL_BIN=<rust-firmware>\rust-ipl\target\x86_64-unknown-uefi\debug\rust-ipl.efi
set RUST_PAYLOAD_BIN=<rust-firmware>\rust-uefi-payload\target\target\debug\rust-uefi-payload
set RUST_FIRMWARE_BIN=<rust-firmware>\rust-firmware-tool\final.bin
cargo run %RESET_VECTOR_BIN% %RUST_IPL_BIN% %RUST_PAYLOAD_BIN% %RUST_FIRMWARE_BIN%
```

## Run

```
set QEMU_BIN="C:\Program Files\qemu\qemu-system-x86_64.exe"
set BIOS_BIN=<rust-firmware>\rust-firmware-tool\final.bin
set SERIAL_LOG=-serial file:serial.log
%QEMU_BIN% -machine q35 -drive if=pflash,format=raw,unit=0,file=%BIOS_BIN%,readonly=on %SERIAL_LOG%
```
