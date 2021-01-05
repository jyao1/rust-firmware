// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use bitfield::Bit;
use bitfield::BitRange;

pub const PCI_CONFIGURATION_ADDRESS_PORT:  u16 = 0xCF8;
pub const PCI_CONFIGURATION_DATA_PORT: u16 = 0xCFC;

#[allow(non_snake_case)]
#[cfg(not(test))]
pub fn PciCf8Read32(bus: u8, device: u8, fnc: u8, reg: u8) -> u32
{
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

#[allow(non_snake_case)]
#[cfg(not(test))]
pub fn PciCf8Write32(bus: u8, device: u8, fnc: u8, reg: u8, value: u32)
{
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

#[allow(non_snake_case)]
#[cfg(not(test))]
pub fn PciCf8Write8(bus: u8, device: u8, fnc: u8, reg: u8, value: u8)
{
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


#[allow(non_snake_case)]
#[cfg(not(test))]
pub fn PciCf8Read8(bus: u8, device: u8, fnc: u8, reg: u8) -> u8
{
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

#[allow(non_snake_case)]
#[cfg(not(test))]
pub fn InitializeAcpiPm()
{
    let mut PmbaAndVal = 0xffffffffu32;
    PmbaAndVal.set_bit_range(15, 7, 0x0u32);
    let PmbaOrVal = 0x600u32;
    let AcpiEnBit = 0x80u32;

    let mut AcpiControlReg = PciCf8Read8(0, 0x1f, 0, 0x44);
    if AcpiControlReg.bit(7) == false {
        //
        // The PEI phase should be exited with fully accessibe ACPI PM IO space:
        // 1. set PMBA
        //
        let res = PciCf8Read32(0, 0x1f, 0, 0x40);
        let res = (res & PmbaAndVal) | PmbaOrVal;
        PciCf8Write32(0, 0x1f, 0, 0x40, res);

        //
        // 2. set PCICMD/IOSE
        //
        let res = PciCf8Read8(0, 0x1f, 0, 0x4);
        let res = res | 0x1;
        PciCf8Write8(0, 0x1f, 0, 0x4, res);

        //
        // 3. set ACPI PM IO enable bit (PMREGMISC:PMIOSE or ACPI_CNTL:ACPI_EN)
        //
        AcpiControlReg.set_bit(7, true);
        PciCf8Write8(0, 0x1f, 0, 0x44, AcpiControlReg);
    }
}