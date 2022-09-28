_BUILD_MODE ?= release
_OUT_DIR := target/x86_64-unknown-uefi/$(_BUILD_MODE)

build:
	RUST_FIRMWARE_FSP_FD_FILE=$(PWD)/FAKE_OVMF.fd \
	EDK2_PATH=$(PWD)/EDK2 \
	cargo xbuild --target x86_64-unknown-uefi --release

.PHONY: fsp
fsp:
	cargo build -p build-fsp --bin build_qemu_fsp_release

assemble:
	RUST_FIRMWARE_TOOL_FSP_M_FILE=$(PWD)/FAKE_OVMF.fd \
	RUST_FIRMWARE_TOOL_FSP_S_FILE=$(PWD)/FAKE_OVMF.fd \
	RUST_FIRMWARE_TOOL_FSP_T_FILE=$(PWD)/FAKE_OVMF.fd \
	cargo run -p rust-firmware-tool -- \
		$(_OUT_DIR)/ResetVector.bin \
		$(_OUT_DIR)/rust_ipl.efi \
		$(_OUT_DIR)/rust-uefi-payload.efi \
		$(_OUT_DIR)/final.bin

qemu:
	qemu-system-x86_64 -m 4G -machine q35 \
		-drive if=pflash,format=raw,unit=0,file=$(_OUT_DIR)/final.bin \
		-serial mon:stdio -nographic -vga none -nic none
