# rust-firmware

A demo for pure rust based firmware.

rust-rust-ipl/ResetVector is derived from [EDKII](https://github.com/tianocore/edk2)

rust-uefi-payload is derived from [rust-hypervisor-firmware](https://github.com/cloud-hypervisor/rust-hypervisor-firmware)


## Quick Start (QemuFsp)

**Note:** If your build host is behind company’s firewall, it is important to set up proxy correctly.

### 1. Build QemuFsp

```
cargo run -p build-fsp --bin build_qemu_fsp_release
```

### 2. Build resetvector rust-ipl and rust-uefi-payload

```
cargo xbuild --target x86_64-unknown-uefi --release
```

### 3. Generate firmware file (use rust-firmware-tool).

```
cargo run -p rust-firmware-tool -- target/x86_64-unknown-uefi/release/ResetVector.bin target/x86_64-unknown-uefi/release/rust_ipl.efi target/x86_64-unknown-uefi/release/rust-uefi-payload.efi target/x86_64-unknown-uefi/release/final.bin
```

### 4. Run final.bin in Qemu

```
qemu-system-x86_64 -m 4G -machine q35 -drive if=pflash,format=raw,unit=0,file=target/x86_64-unknown-uefi/release/final.bin -serial mon:stdio -nographic
```

## Known limitation
This package is only the sample code to show the concept. It does not have a full validation such as robustness functional test and fuzzing test. It does not meet the production quality yet. Any codes including the API definition, the libary and the drivers are subject to change.