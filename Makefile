_FSP := QemuFsp/BuildFsp/QEMU_FSP_RELEASE.fd
_BUILD_MODE ?= release
_OUT_DIR := target/x86_64-unknown-uefi/$(_BUILD_MODE)
_APP_DIR := ../uefi-rs/

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

# https://en.wikibooks.org/wiki/QEMU/Devices/Virtio
# https://www.qemu.org/2021/01/19/virtio-blk-scsi-configuration/

qemu:
	qemu-system-x86_64 -m 4G -machine q35 \
		-drive if=pflash,format=raw,unit=0,file=$(_OUT_DIR)/final.bin \
		-drive format=raw,file=$(_APP_DIR)/boot.fat,if=none,id=drive0 \
		-device virtio-scsi-pci,id=scsi \
    -device scsi-hd,drive=boot \
		-drive format=raw,file=fat:rw:boot,if=none,id=boot \
		-serial mon:stdio -nographic -vga none -nic none \
		-kernel ~/Projects/Fiedka/minilb

qemu-payload:
	qemu-system-x86_64 -nodefaults -machine q35 -smp 4 -m 256M --enable-kvm \
		-serial mon:stdio -vga std -nic none \
		-device virtio-rng-pci \
		-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
		-drive if=pflash,format=raw,unit=0,file=$(_OUT_DIR)/final.bin \
		-drive format=raw,file=fat:rw:$(_APP_DIR)/target/x86_64-unknown-uefi/debug/esp
##	-nic user,model=e1000,net=192.168.17.0/24,tftp=$(_APP_DIR)/uefi-test-runner/tftp/,bootfile=fake-boot-file
##	-drive if=pflash,format=raw,readonly=off,file=/tmp/.tmpnjEpw1/ovmf_vars \
##	-drive format=raw,file=/tmp/.tmpnjEpw1/test_disk.fat.img \
##	-serial stdio -serial pipe:/tmp/.tmpnjEpw1/serial \
##	-qmp pipe:/tmp/.tmpnjEpw1/qemu-monitor \
