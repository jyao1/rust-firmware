_BUILD_MODE ?= release
_OUT_DIR := target/x86_64-unknown-uefi/$(_BUILD_MODE)
_FSP := QemuFsp/BuildFsp/QEMU_FSP_RELEASE.fd

all: fsp build assemble qemu

run: all qemu

.PHONY: fsp
fsp:
	cargo run -p build-fsp --bin build_qemu_fsp_release

build:
	RUST_FIRMWARE_FSP_FD_FILE=$(PWD)/$(_FSP) \
	EDK2_PATH=$(PWD)/QemuFsp \
	cargo xbuild --target x86_64-unknown-uefi --release

assemble:
	RUST_FIRMWARE_FSP_FD_FILE=$(PWD)/$(_FSP) \
	cargo run -p rust-firmware-tool -- \
		$(_OUT_DIR)/ResetVector.bin \
		$(_OUT_DIR)/rust_ipl.efi \
		$(_OUT_DIR)/rust-uefi-payload.efi \
		$(_OUT_DIR)/final.bin

qemu:
	qemu-system-x86_64 -m 4G -machine q35 \
		-drive if=pflash,format=raw,unit=0,file=$(_OUT_DIR)/final.bin \
		-serial mon:stdio -nographic -vga none -nic none
