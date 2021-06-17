# Build Rust Firmware

## Tools

1. Install [RUST](https://www.rust-lang.org/)

1.1. Intall xbuild

```
cargo install cargo-xbuild
```

2. Install [NASM](https://www.nasm.us/)

## Build (in linux or git bash)

### Generate IPL and Reset Vector and rust-uefi-payload

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
