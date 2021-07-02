# Build Rust Firmware

## Tools

1. Install [RUST](https://www.rust-lang.org/)

1.1. Intall xbuild

```
cargo install cargo-xbuild
```

2. Install [NASM](https://www.nasm.us/)

## Build (in linux or git bash)

### How to build QemuFsp

run `cargo build -p build-fsp --bin build_qemu_fsp_release`

**note:** if you behind proxy, you should set proxy for wget/git
**note:** `http_proxy` and `https_proxy` may not work for wget. you can set `WGETRC` to specify the config file for it.

### Generate IPL and Reset Vector and rust-uefi-payload

Before build environment variable should be set.

`EDK2_PATH` should be set to EDK2's source directory, if not set, it will use defaut one.
`RUST_FIRMWARE_FSP_FD_FILE` should be set to [FSP_RELEASE_SOME_NAME.fd], it not set, it will use default one.

```
export EDK2_PATH=[FULL_PATH]\edk2
export RUST_FIRMWARE_FSP_FD_FILE=[FULL_FSP_PATH]\QEMU_FSP_RELEASE.fd
```

Build reset vector,  rust_ipl and rust-uefi-payload

```
cargo xbuild --target x86_64-unknown-uefi --release

export RESET_VECTOR_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/ResetVector.bin
export RUST_IPL_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust_ipl.efi
export RUST_PAYLOAD_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust-uefi-payload.efi
```

### Generate firmware file (use rust-firmware-tool).

```
export RUST_FIRMWARE_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/final.bin
cargo run -p rust-firmware-tool -- $RESET_VECTOR_BIN $RUST_IPL_BIN $RUST_PAYLOAD_BIN $RUST_FIRMWARE_BIN
```

## Run (in linux or git bash)

1. install qemu

**note**: git bash doesn't contains qemu, you should install it.

2. run with generated firmware

```
qemu-system-x86_64 -m 3072 -machine q35 -drive if=pflash,format=raw,unit=0,file=$RUST_FIRMWARE_BIN -serial mon:stdio -nographic
```
