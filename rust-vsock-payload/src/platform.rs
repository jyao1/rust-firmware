// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/// PCI virtio vsock device init

pub fn init() {
    // TBD: hard code need be removed
    let pci_device = fw_pci::PciDevice::new(0, 2, 0);
    pci_device.write_u8(0x4, 0x7);
    pci_device.write_u8(0x5, 0x4);
    pci_device.write_u8(0x10, 0xe1);
    pci_device.write_u8(0x11, 0x60);
    pci_device.write_u8(0x15, 0x20);
    pci_device.write_u8(0x17, 0xc0);
    pci_device.write_u32(0x20, 0xfe000008);
    pci_device.write_u32(0x24, 0);
    pci_device.write_u8(0x3C, 0x0a);

    dump_pci(&pci_device);
}

fn dump_pci(pci_device: &fw_pci::PciDevice) {
    log::info!(
        "pci: {:02X}:{:02X}:{:02X}\n",
        pci_device.bus,
        pci_device.device,
        pci_device.func
    );

    let command = pci_device.read_u16(0x4);
    let status = pci_device.read_u16(0x6);
    rust_ipl_log::write_args(format_args!(
        "bit  \t fedcba9876543210\nstate\t {:016b}\ncommand\t {:016b}\n",
        status, command
    ));
    dump_bar(0x10, pci_device);
    dump_bar(0x14, pci_device);
    dump_bar(0x18, pci_device);
    dump_bar(0x1C, pci_device);
    dump_bar(0x20, pci_device);
    dump_bar(0x24, pci_device);

    dump_pic_16_bytes(0x0, pci_device);
    dump_pic_16_bytes(0x10, pci_device);
    dump_pic_16_bytes(0x20, pci_device);
    dump_pic_16_bytes(0x30, pci_device);
}

fn dump_bar(offset: u8, pci_devide: &fw_pci::PciDevice) {
    let bar0 = pci_devide.read_u32(offset);
    rust_ipl_log::write_args(format_args!(
        "bar offset {:X}, value: {:#08x}\n",
        offset, bar0
    ));
}

fn dump_pic_16_bytes(offset: u8, pci_device: &fw_pci::PciDevice) {
    rust_ipl_log::write_args(format_args!("{:02x}:", offset));
    for i in 0..16 {
        let res0 = pci_device.read_u8(offset + i);
        rust_ipl_log::write_args(format_args!(" {:02x}", res0));
    }
    rust_ipl_log::write_args(format_args!("\n"));
}
