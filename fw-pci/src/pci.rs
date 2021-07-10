// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use bitfield::Bit;
use bitfield::BitRange;

pub const PCI_CONFIGURATION_ADDRESS_PORT: u16 = 0xCF8;
pub const PCI_CONFIGURATION_DATA_PORT: u16 = 0xCFC;

pub fn pci_cf8_read32(bus: u8, device: u8, fnc: u8, reg: u8) -> u32 {
    let data = u32::from(bus) << 16;
    let data = data | u32::from(device) << 11;
    let data = data | u32::from(fnc) << 8;
    let data = data | u32::from(reg & 0xfc);
    let data = data | 1u32 << 31;

    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        x86::io::inl(PCI_CONFIGURATION_DATA_PORT)
    }
}


pub fn pci_cf8_write32(bus: u8, device: u8, fnc: u8, reg: u8, value: u32) {
    let data = u32::from(bus) << 16;
    let data = data | u32::from(device) << 11;
    let data = data | u32::from(fnc) << 8;
    let data = data | u32::from(reg & 0xfc);
    let data = data | 1u32 << 31;

    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        x86::io::outl(PCI_CONFIGURATION_DATA_PORT, value);
    }
}


pub fn pci_cf8_write8(bus: u8, device: u8, fnc: u8, reg: u8, value: u8) {
    let data = u32::from(bus) << 16;
    let data = data | u32::from(device) << 11;
    let data = data | u32::from(fnc) << 8;
    let data = data | u32::from(reg & 0xfc);
    let data = data | 1u32 << 31;

    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        x86::io::outb(PCI_CONFIGURATION_DATA_PORT + (data & 3) as u16, value);
    }
}


pub fn pci_cf8_read8(bus: u8, device: u8, fnc: u8, reg: u8) -> u8 {
    let data = u32::from(bus) << 16;
    let data = data | u32::from(device) << 11;
    let data = data | u32::from(fnc) << 8;
    let data = data | u32::from(reg & 0xfc);
    let data = data | 1u32 << 31;

    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        x86::io::inb(PCI_CONFIGURATION_DATA_PORT + (data & 3) as u16)
    }
}


pub fn initialize_acpi_pm() {
    let mut pmba_and_val = 0xffffffffu32;
    pmba_and_val.set_bit_range(15, 7, 0x0u32);
    let pmba_or_val = 0x600u32;
    let _acpi_en_bit = 0x80u32;

    let mut acpi_control_reg = pci_cf8_read8(0, 0x1f, 0, 0x44);
    if !acpi_control_reg.bit(7) {
        //
        // The PEI phase should be exited with fully accessibe ACPI PM IO space:
        // 1. set PMBA
        //
        let res = pci_cf8_read32(0, 0x1f, 0, 0x40);
        let res = (res & pmba_and_val) | pmba_or_val;
        pci_cf8_write32(0, 0x1f, 0, 0x40, res);

        //
        // 2. set PCICMD/IOSE
        //
        let res = pci_cf8_read8(0, 0x1f, 0, 0x4);
        let res = res | 0x1;
        pci_cf8_write8(0, 0x1f, 0, 0x4, res);

        //
        // 3. set ACPI PM IO enable bit (PMREGMISC:PMIOSE or ACPI_CNTL:ACPI_EN)
        //
        acpi_control_reg.set_bit(7, true);
        pci_cf8_write8(0, 0x1f, 0, 0x44, acpi_control_reg);
    }
}

pub fn pci_ex_bar_initialization() {
    // PcdPciExpressBaseAddress TBD
    let pci_exbar_base = 0x80000000u64;

    //
    // Clear the PCIEXBAREN bit first, before programming the high register.
    //
    pci_cf8_write32(0, 0, 0, 0x60, 0);

    //
    // Program the high register. Then program the low register, setting the
    // MMCONFIG area size and enabling decoding at once.
    //
    log::info!("pci_exbar_base {:x}\n", pci_exbar_base);
    log::info!(
        "pci_exbar_base {:x}, {:x}\n",
        (pci_exbar_base >> 32) as u32,
        (pci_exbar_base << 32 >> 32 | 0x1) as u32
    );
    pci_cf8_write32(0, 0, 0, 0x64, (pci_exbar_base >> 32) as u32);
    pci_cf8_write32(0, 0, 0, 0x60, (pci_exbar_base << 32 >> 32 | 0x1) as u32);
}


pub fn init_pci() {
    pci_cf8_write32(0, 3, 0, 0x14, 0xC1085000);
    pci_cf8_write32(0, 3, 0, 0x20, 0xC200000C);
    pci_cf8_write32(0, 3, 0, 0x24, 0x00000008);
    pci_cf8_write8(0, 3, 0, 0x4, 0x07);
}

pub fn virt_io_blk() {
    let base: usize = 0x8C2000000usize;
    use core::intrinsics::volatile_store;

    log::info!("VIRTIO_STATUS_RESET\n");
    unsafe {
        volatile_store((base + 0x14usize) as *mut u32, 0u32);
    }
    log::info!("VIRTIO_STATUS_ACKNOWLEDGE\n");
    unsafe {
        volatile_store((base + 0x14usize) as *mut u32, 1u32);
    }
    log::info!("VIRTIO_STATUS_DRIVER\n");
    unsafe {
        volatile_store((base + 0x14usize) as *mut u32, 2u32);
    }
}

fn pci_cf8_read16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    assert_eq!(offset % 2, 0);
    (pci_cf8_read32(bus, device, func, offset & !3) >> ((offset % 4) * 8)) as u16
}

fn get_device_details(bus: u8, device: u8, func: u8) -> (u16, u16) {
    (
        pci_cf8_read16(bus, device, func, 0),
        pci_cf8_read16(bus, device, func, 2),
    )
}

pub fn print_bus() {
    const MAX_DEVICES:u8 = 32;
    const INVALID_VENDOR_ID: u16 = 0xffff;

    for device in 0..MAX_DEVICES {
        let (vendor_id, device_id) = get_device_details(0, device, 0);
        if vendor_id == INVALID_VENDOR_ID {
            continue;
        }
        log::info!(
            "Found PCI device vendor={:x} device={:x} in slot={}\n",
            vendor_id,
            device_id,
            device
        );
    }
}
