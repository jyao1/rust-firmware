// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use bitfield::Bit;
use bitfield::BitRange;

use bitflags::bitflags;

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

fn get_device_details(bus: u8, device: u8, func: u8) -> (u16, u16) {
    let config_data = ConfigSpacePciEx::read::<u32>(bus, device, func, 0);
    (
        (config_data & 0xffff) as u16,
        ((config_data & 0xffff0000) >> 0x10) as u16,
    )
}

pub fn print_bus() {
    const MAX_DEVICES: u8 = 32;
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

/// Configure Space Access Mechanism #1

/// 32-bit I/O locations  CONFIG_ADDRESS (0xCF8)
/// 0-7     register offset
/// 8-10    funtion number
/// 11-15   device number
/// 16-23   bus number
/// 24-30   reserved
/// 31      enable bit
pub type ConfigAddress = u32;

/// Configure Space
pub struct ConfigSpace;

impl ConfigSpace {
    pub fn read32(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
        assert!(offset % 4 == 0);
        let config_address = Self::get_config_address(bus, device, func, offset);
        unsafe {
            x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, config_address);
            x86::io::inl(PCI_CONFIGURATION_DATA_PORT)
        }
    }

    pub fn write32(bus: u8, device: u8, func: u8, offset: u8, config_data: u32) {
        assert!(offset % 4 == 0);
        let config_address = Self::get_config_address(bus, device, func, offset);
        unsafe {
            x86::io::outl(PCI_CONFIGURATION_ADDRESS_PORT, config_address);
            x86::io::outl(PCI_CONFIGURATION_DATA_PORT, config_data);
        }
    }

    pub fn read16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
        assert!(offset & 0b01 == 0);
        let config_data = ConfigSpace::read32(bus, device, func, offset & 0b1111_1100);
        ((config_data >> ((offset & 0b10) << 3)) & 0xFFFF) as u16
    }

    pub fn write16(bus: u8, device: u8, func: u8, offset: u8, config_data: u16) {
        assert!(offset & 0b01 == 0);
        let old_config_data = ConfigSpace::read32(bus, device, func, offset);
        let dest = (offset & 0b010) << 3; // 0 0x10
        let mask = 0xffffu32 << dest;
        let new_config_data = (config_data as u32) << dest | (old_config_data & !mask);
        ConfigSpace::write32(bus, device, func, offset, new_config_data)
    }

    pub fn read8(bus: u8, device: u8, func: u8, offset: u8) -> u8 {
        let config_data = ConfigSpace::read32(bus, device, func, offset & 0b1111_1100);
        ((config_data >> ((offset as usize & 0b11) << 3)) & 0xFF) as u8
    }

    pub fn write8(bus: u8, device: u8, func: u8, offset: u8, config_data: u8) {
        assert!(offset & 0b01 == 0);
        let old_config_data = ConfigSpace::read32(bus, device, func, offset);
        let dest = (offset & 0b011) << 3; // 0 0x8 0x10 0x18
        let mask = 0xffu32 << dest;
        let new_config_data = (config_data as u32) << dest | (old_config_data & !mask);
        ConfigSpace::write32(bus, device, func, offset, new_config_data)
    }

    /// Get vendor_id and device_id
    pub fn get_device_details(bus: u8, device: u8, func: u8) -> (u16, u16) {
        let config_data = ConfigSpacePciEx::read::<u32>(bus, device, func, 0);
        (
            (config_data & 0xffff) as u16,
            ((config_data & 0xffff0000) >> 0x10) as u16,
        )
    }

    fn get_config_address(bus: u8, device: u8, func: u8, offset: u8) -> ConfigAddress {
        let offset = offset & 0b1111_1100;
        let func = func & 0b0000_0111;
        let device = device & 0b0001_1111;

        (1 << 31)
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((func as u32) << 8)
            | offset as u32
    }
}

pub struct ConfigSpacePciEx;
const PCI_EX_BAR_BASE_ADDRESS: u64 = 0xE0000000u64;
impl ConfigSpacePciEx {
    pub fn read<T>(bus: u8, device: u8, func: u8, offset: u16) -> T {
        assert!(offset < 0x1000);
        assert!(offset % core::mem::size_of::<T>() as u16 == 0);
        let addr = PCI_EX_BAR_BASE_ADDRESS
            + ((bus as u64) << 20)
            + ((device as u64) << 15)
            + ((func as u64) << 12)
            + offset as u64;
        unsafe { core::ptr::read_volatile(addr as *const T) }
    }
    pub fn write<T>(bus: u8, device: u8, func: u8, offset: u16, value: T) {
        assert!(offset < 0x1000);
        assert!(offset % core::mem::size_of::<T>() as u16 == 0);
        let addr = PCI_EX_BAR_BASE_ADDRESS
            + ((bus as u64) << 20)
            + ((device as u64) << 15)
            + ((func as u64) << 12)
            + offset as u64;
        unsafe { core::ptr::write_volatile(addr as *mut T, value) }
    }
}

/// CommonHeader to all PCI Header Type
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PciDeviceCommonHeader {
    pub device_id: u16,
    pub vendor_id: u16,
    pub status: Status,
    pub command: Command,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub bist: u8,
    pub header_type: u8,
    pub latency_time: u8,
    pub cache_line_size: u8,
}

bitflags! {
    #[derive(Default)]
    pub struct HeaderType: u8 {
        const MF   = 0b10000000;
        const STANDARD = 0x0;
        const PCI2PCI_BRIDGE = 0x1;
        const PCI2CARDBUS_BRIDGE = 0x2;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Status: u16 {
        const RESERVED_0                = 0x0001;
        const RESERVED_1                = 0x0002;
        const RESERVED_2                = 0x0004;
        const INTERRUPT_STATUS          = 0x0008;
        const CAPABILITIES_LIST         = 0x0010;
        const MHZ66_CAPABLE             = 0x0020;
        const RESERVED_6                = 0x0040;
        const FAST_BACK_TO_BACK_CAPABLE = 0x0080;
        const MASTER_DATA_PARITY_ERROR  = 0x0100;
        const DEVSEL_MEDIUM_TIMING      = 0x0200;
        const DEVSEL_SLOW_TIMING        = 0x0400;
        const SIGNALED_TARGET_ABORT     = 0x0800;
        const RECEIVED_TARGET_ABORT     = 0x1000;
        const RECEIVED_MASTER_ABORT     = 0x2000;
        const SIGNALED_SYSTEM_ERROR     = 0x4000;
        const DETECTED_PARITY_ERROR     = 0x8000;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Command: u16 {
        const IO_SPACE                  = 0x0001;
        const MEMORY_SPACE              = 0x0002;
        const BUS_MASTER                = 0x0004;
        const SPECIAL_CYCLES            = 0x0008;
        const MWI_ENABLE                = 0x0010;
        const VGA_PALETTE_SNOOP         = 0x0020;
        const PARITY_ERROR_RESPONSE     = 0x0040;
        const STEPPING_CONTROL          = 0x0080;
        const SERR_ENABLE               = 0x0100;
        const FAST_BACK_TO_BACK_ENABLE  = 0x0200;
        const INTERRUPT_DISABLE         = 0x0400;
        const RESERVED_11               = 0x0800;
        const RESERVED_12               = 0x1000;
        const RESERVED_13               = 0x2000;
        const RESERVED_14               = 0x4000;
        const RESERVED_15               = 0x8000;
    }
}

#[derive(Default)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub func: u8,

    // Pci Device Header
    pub common_header: PciDeviceCommonHeader,
    pub bars: [PciBar; 6],
    pub cardbus_cis_pointer: u32,
    pub subsystem_id: u16,
    pub subsystem_vendor_id: u16,
    pub expansion_rom_base_address: u32,
    pub reserved1: u16,
    pub reserved2: u8,
    pub capabilities_pointer: u8,
    pub reserved3: u32,
    pub max_latency: u8,
    pub min_grant: u8,
    pub interrupt_pin: u8,
    pub interrup_line: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum PciBarType {
    Unused,
    MemorySpace32,
    MemorySpace64,
    IoSpace,
}

impl Default for PciBarType {
    fn default() -> Self {
        PciBarType::Unused
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Copy)]
pub struct PciBar {
    pub address: u64,
    pub bar_type: PciBarType,
}

impl PciDevice {
    pub fn new(bus: u8, device: u8, func: u8) -> PciDevice {
        PciDevice {
            bus,
            device,
            func,
            common_header: PciDeviceCommonHeader::default(),
            ..Default::default()
        }
    }

    pub fn init(&mut self) {
        let (vendor_id, device_id) =
            ConfigSpace::get_device_details(self.bus, self.device, self.func);
        self.common_header.vendor_id = vendor_id;
        self.common_header.device_id = device_id;
        let command = self.read_u16(0x4);
        let status = self.read_u16(0x6);
        log::info!(
            "PCI Device: {}:{}.{} {:x}:{:x}\nbit  \t fedcba9876543210\nstate\t {:016b}\ncommand\t {:016b}\n",
            self.bus,
            self.device,
            self.func,
            self.common_header.vendor_id,
            self.common_header.device_id,
            status,
            command,
        );

        let mut current_bar_offset = 0x10;
        let mut current_bar = 0;

        //0x24 offset is last bar
        while current_bar_offset < 0x24 {
            let bar = self.read_u32(current_bar_offset);

            // lsb is 1 for I/O space bars
            if bar & 1 == 1 {
                self.bars[current_bar].bar_type = PciBarType::IoSpace;
                self.bars[current_bar].address = u64::from(bar & 0xffff_fffc);
            } else {
                // bits 2-1 are the type 0 is 32-but, 2 is 64 bit
                match bar >> 1 & 3 {
                    0 => {
                        self.bars[current_bar].bar_type = PciBarType::MemorySpace32;
                        self.bars[current_bar].address = u64::from(bar & 0xffff_fff0);
                    }
                    2 => {
                        self.bars[current_bar].bar_type = PciBarType::MemorySpace64;
                        self.bars[current_bar].address = u64::from(bar & 0xffff_fff0);
                        current_bar_offset += 4;

                        let bar = self.read_u32(current_bar_offset);
                        self.bars[current_bar].address += u64::from(bar) << 32;
                    }
                    _ => panic!("Unsupported BAR type"),
                }
            }

            current_bar += 1;
            current_bar_offset += 4;
        }
        for bar in &self.bars {
            log::info!("Bar: type={:?} address={:x}\n", bar.bar_type, bar.address);
        }
    }
    pub fn read_u64(&self, offset: u8) -> u64 {
        ConfigSpacePciEx::read::<u64>(self.bus, self.device, self.func, offset as u16)
    }

    pub fn read_u32(&self, offset: u8) -> u32 {
        ConfigSpacePciEx::read::<u32>(self.bus, self.device, self.func, offset as u16)
    }

    pub fn read_u16(&self, offset: u8) -> u16 {
        ConfigSpacePciEx::read::<u16>(self.bus, self.device, self.func, offset as u16)
    }

    pub fn read_u8(&self, offset: u8) -> u8 {
        ConfigSpacePciEx::read::<u8>(self.bus, self.device, self.func, offset as u16)
    }

    pub fn write_u32(&self, offset: u8, value: u32) {
        ConfigSpacePciEx::write::<u32>(self.bus, self.device, self.func, offset as u16, value)
    }

    pub fn write_u16(&self, offset: u8, value: u16) {
        ConfigSpacePciEx::write::<u16>(self.bus, self.device, self.func, offset as u16, value)
    }

    pub fn write_u8(&self, offset: u8, value: u8) {
        ConfigSpacePciEx::write::<u8>(self.bus, self.device, self.func, offset as u16, value)
    }
}
