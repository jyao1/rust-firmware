// Copyright (c) 2020 Intel Corporation
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

    let mut result: u32 = 0u32;
    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        result = x86::io::inl(PCI_CONFIGURATION_DATA_PORT);
    }
    result
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

    let mut result = 0u8;
    unsafe {
        x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, data);
        result = x86::io::inb(PCI_CONFIGURATION_DATA_PORT + (data & 3) as u16);
    }
    result
}


pub fn initialize_acpi_pm() {
    let mut pmba_and_val = 0xffffffffu32;
    pmba_and_val.set_bit_range(15, 7, 0x0u32);
    let pmba_or_val = 0x600u32;
    let acpi_en_bit = 0x80u32;

    let mut acpi_control_reg = pci_cf8_read8(0, 0x1f, 0, 0x44);
    if acpi_control_reg.bit(7) == false {
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
