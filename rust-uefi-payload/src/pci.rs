// Copyright © 2019 Intel Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused)]

#[macro_use]
use fw_logger::*;

use x86_64::instructions::port::Port;

// #[cfg(not(test))]
use crate::virtio::Error as VirtioError;
// #[cfg(not(test))]
use crate::virtio::VirtioTransport;

use crate::mem;

// #[cfg(not(test))]
const CONFIG_ADDRESS: u16 = 0xcf8;
// #[cfg(not(test))]
const CONFIG_DATA: u16 = 0xcfc;

// #[cfg(not(test))]
const MAX_DEVICES: u8 = 32;
// #[cfg(not(test))]
const MAX_FUNCTIONS: u8 = 8;

// #[cfg(not(test))]
const INVALID_VENDOR_ID: u16 = 0xffff;

// #[cfg(not(test))]
fn pci_config_read_u32(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    assert_eq!(offset % 4, 0);
    assert!(device < MAX_DEVICES);
    assert!(func < MAX_FUNCTIONS);

    let addr = u32::from(bus) << 16; // bus bits 23-16
    let addr = addr | u32::from(device) << 11; // slot/device bits 15-11
    let addr = addr | u32::from(func) << 8; // function bits 10-8
    let addr = addr | u32::from(offset & 0xfc); // register 7-0
    let addr = addr | 1u32 << 31; // enable bit 31

    unsafe {
        let mut config_address_port: Port<u32> = unsafe { Port::new(CONFIG_ADDRESS) };
        config_address_port.write(addr);

        let mut config_data_port: Port<u32> = unsafe { Port::new(CONFIG_DATA) };

        config_data_port.read()
    }
}

// #[cfg(not(test))]
fn pci_config_read_u8(bus: u8, device: u8, func: u8, offset: u8) -> u8 {
    (pci_config_read_u32(bus, device, func, offset & !3) >> ((offset % 4) * 8)) as u8
}

// #[cfg(not(test))]
fn pci_config_read_u16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    assert_eq!(offset % 2, 0);
    (pci_config_read_u32(bus, device, func, offset & !3) >> ((offset % 4) * 8)) as u16
}

// #[cfg(not(test))]
fn pci_config_write_u32(bus: u8, device: u8, func: u8, offset: u8, value: u32) -> () {
    assert_eq!(offset % 4, 0);
    assert!(device < MAX_DEVICES);
    assert!(func < MAX_FUNCTIONS);

    let addr = u32::from(bus) << 16; // bus bits 23-16
    let addr = addr | u32::from(device) << 11; // slot/device bits 15-11
    let addr = addr | u32::from(func) << 8; // function bits 10-8
    let addr = addr | u32::from(offset & 0xfc); // register 7-0
    let addr = addr | 1u32 << 31; // enable bit 31

    unsafe {
        let mut config_address_port: Port<u32> = unsafe { Port::new(CONFIG_ADDRESS) };
        config_address_port.write(addr);

        let mut config_data_port: Port<u32> = unsafe { Port::new(CONFIG_DATA) };

        config_data_port.write(value)
    }
}

// #[cfg(not(test))]
fn pci_config_write_u16(bus: u8, device: u8, func: u8, offset: u8, value: u16) -> () {
    assert_eq!(offset % 2, 0);
    assert!(device < MAX_DEVICES);
    assert!(func < MAX_FUNCTIONS);

    let addr = u32::from(bus) << 16; // bus bits 23-16
    let addr = addr | u32::from(device) << 11; // slot/device bits 15-11
    let addr = addr | u32::from(func) << 8; // function bits 10-8
    let addr = addr | u32::from(offset & 0xfe); // register 7-0
    let addr = addr | 1u32 << 31; // enable bit 31

    unsafe {
        let mut config_address_port: Port<u32> = unsafe { Port::new(CONFIG_ADDRESS) };
        config_address_port.write(addr);

        let mut config_data_port: Port<u16> = unsafe { Port::new(CONFIG_DATA) };

        config_data_port.write(value)
    }
}

// #[cfg(not(test))]
fn pci_config_write_u8(bus: u8, device: u8, func: u8, offset: u8, value: u8) -> () {
    assert!(device < MAX_DEVICES);
    assert!(func < MAX_FUNCTIONS);

    let addr = u32::from(bus) << 16; // bus bits 23-16
    let addr = addr | u32::from(device) << 11; // slot/device bits 15-11
    let addr = addr | u32::from(func) << 8; // function bits 10-8
    let addr = addr | u32::from(offset); // register 7-0
    let addr = addr | 1u32 << 31; // enable bit 31

    unsafe {
        let mut config_address_port: Port<u32> = unsafe { Port::new(CONFIG_ADDRESS) };
        config_address_port.write(addr);

        let mut config_data_port: Port<u8> = unsafe { Port::new(CONFIG_DATA) };

        config_data_port.write(value)
    }
}

// #[cfg(not(test))]
fn get_device_details(bus: u8, device: u8, func: u8) -> (u16, u16) {
    (
        pci_config_read_u16(bus, device, func, 0),
        pci_config_read_u16(bus, device, func, 2),
    )
}

// #[cfg(not(test))]
pub fn print_bus() {
    for device in 0..MAX_DEVICES {
        let (vendor_id, device_id) = get_device_details(0, device, 0);
        if vendor_id == INVALID_VENDOR_ID {
            continue;
        }
        log!(
            "Found PCI device vendor={:x} device={:x} in slot={}\n",
            vendor_id,
            device_id,
            device
        );
    }
}

// #[cfg(not(test))]
pub fn search_bus(target_vendor_id: u16, target_device_id: u16) -> Option<PciDevice> {
    for device in 0..MAX_DEVICES {
        let (vendor_id, device_id) = get_device_details(0, device, 0);
        if vendor_id == target_vendor_id && device_id == target_device_id {
            return Some(PciDevice::new(0, device, 0));
        }
    }
    None
}

// #[cfg(not(test))]
#[derive(Default)]
pub struct PciDevice {
    bus: u8,
    pub device: u8,
    pub func: u8,
    bars: [PciBar; 6],
    vendor_id: u16,
    device_id: u16,
}

// #[cfg(not(test))]
#[derive(Debug)]
enum PciBarType {
    Unused,
    MemorySpace32,
    MemorySpace64,
    IoSpace,
}

// #[cfg(not(test))]
impl Default for PciBarType {
    fn default() -> Self {
        PciBarType::Unused
    }
}

// #[cfg(not(test))]
#[derive(Default)]
struct PciBar {
    bar_type: PciBarType,
    address: u64,
}

// #[cfg(not(test))]
impl PciDevice {
    fn new(bus: u8, device: u8, func: u8) -> PciDevice {
        PciDevice {
            bus,
            device,
            func,
            ..Default::default()
        }
    }

    fn config_read_u8(&self, offset: u8) -> u8 {
        pci_config_read_u8(self.bus, self.device, self.func, offset)
    }

    fn config_read_u16(&self, offset: u8) -> u16 {
        pci_config_read_u16(self.bus, self.device, self.func, offset)
    }

    fn config_read_u32(&self, offset: u8) -> u32 {
        pci_config_read_u32(self.bus, self.device, self.func, offset)
    }

    #[allow(unused)]
    fn config_write_u8(&self, offset: u8, value: u8) -> () {
        pci_config_write_u8(self.bus, self.device, self.func, offset, value)
    }

    #[allow(unused)]
    fn config_write_u16(&self, offset: u8, value: u16) -> () {
        pci_config_write_u16(self.bus, self.device, self.func, offset, value)
    }

    #[allow(unused)]
    fn config_write_u32(&self, offset: u8, value: u32) -> () {
        pci_config_write_u32(self.bus, self.device, self.func, offset, value)
    }

    fn init(&mut self) {
        let (vendor_id, device_id) = get_device_details(self.bus, self.device, self.func);

        self.vendor_id = vendor_id;
        self.device_id = device_id;

        log!(
            "PCI Device: {}:{}.{} {:x}:{:x}\n",
            self.bus,
            self.device,
            self.func,
            self.vendor_id,
            self.device_id
        );

        let mut current_bar_offset = 0x10;
        let mut current_bar = 0;

        //0x24 offset is last bar
        while current_bar_offset < 0x24 {
            #[allow(clippy::blacklisted_name)]
            let bar = self.config_read_u32(current_bar_offset);

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

                        #[allow(clippy::blacklisted_name)]
                        let bar = self.config_read_u32(current_bar_offset);
                        self.bars[current_bar].address += u64::from(bar) << 32;
                    }
                    _ => panic!("Unsupported BAR type"),
                }
            }

            current_bar += 1;
            current_bar_offset += 4;
        }

        #[allow(clippy::blacklisted_name)]
        for bar in &self.bars {
            log!("Bar: type={:?} address={:x}\n", bar.bar_type, bar.address);
        }
    }
}

// #[cfg(not(test))]
#[allow(clippy::enum_variant_names)]
enum VirtioPciCapabilityType {
    CommonConfig = 1,
    NotifyConfig = 2,
    #[allow(unused)]
    IsrConfig = 3,
    #[allow(unused)]
    DeviceConfig = 4,
    #[allow(unused)]
    PciConfig = 5,
}

// #[cfg(not(test))]
#[derive(Default)]
pub struct VirtioPciTransport {
    device: PciDevice,
    region: mem::MemoryRegion,               // common configuration region
    notify_region: mem::MemoryRegion,        // notify region
    notify_off_multiplier: u32,              // from notify config cap
    device_config_region: mem::MemoryRegion, // device specific region
}

// #[cfg(not(test))]
impl VirtioPciTransport {
    pub fn new(device: PciDevice) -> VirtioPciTransport {
        VirtioPciTransport {
            device,
            ..Default::default()
        }
    }
}
// Common Configuration registers:
/// le32 device_feature_select;     // 0x00 // read-write
/// le32 device_feature;            // 0x04 // read-only for driver
/// le32 driver_feature_select;     // 0x08 // read-write
/// le32 driver_feature;            // 0x0C // read-write
/// le16 msix_config;               // 0x10 // read-write
/// le16 num_queues;                // 0x12 // read-only for driver
/// u8 device_status;               // 0x14 // read-write (driver_status)
/// u8 config_generation;           // 0x15 // read-only for driver
/// ** About a specific virtqueue.
/// le16 queue_select;              // 0x16 // read-write
/// le16 queue_size;                // 0x18 // read-write, power of 2, or 0.
/// le16 queue_msix_vector;         // 0x1A // read-write
/// le16 queue_enable;              // 0x1C // read-write (Ready)
/// le16 queue_notify_off;          // 0x1E // read-only for driver
/// le64 queue_desc;                // 0x20 // read-write
/// le64 queue_avail;               // 0x28 // read-write
/// le64 queue_used;                // 0x30 // read-write

// #[cfg(not(test))]
impl VirtioTransport for VirtioPciTransport {
    fn init(&mut self, _device_type: u32) -> Result<(), VirtioError> {
        self.device.init();

        // Read status register
        let status = self.device.config_read_u16(0x06);

        // bit 4 of status is capability bit
        if status & 1 << 4 == 0 {
            log!("No capabilities detected\n");
            return Err(VirtioError::VirtioUnsupportedDevice);
        }

        // capabilities list offset is at 0x34
        let mut cap_next = self.device.config_read_u8(0x34);

        while cap_next < 0xff && cap_next > 0 {
            // vendor specific capability
            if self.device.config_read_u8(cap_next) == 0x09 {
                // These offsets are into the following structure:
                // struct virtio_pci_cap {
                //         u8 cap_vndr;    /* Generic PCI field: PCI_CAP_ID_VNDR */
                //         u8 cap_next;    /* Generic PCI field: next ptr. */
                //         u8 cap_len;     /* Generic PCI field: capability length */
                //         u8 cfg_type;    /* Identifies the structure. */
                //         u8 bar;         /* Where to find it. */
                //         u8 padding[3];  /* Pad to full dword. */
                //         le32 offset;    /* Offset within bar. */
                //         le32 length;    /* Length of the structure, in bytes. */
                // };
                let cfg_type = self.device.config_read_u8(cap_next + 3);
                #[allow(clippy::blacklisted_name)]
                let bar = self.device.config_read_u8(cap_next + 4);
                let offset = self.device.config_read_u32(cap_next + 8);
                let length = self.device.config_read_u32(cap_next + 12);

                if cfg_type == VirtioPciCapabilityType::CommonConfig as u8 {
                    self.region = mem::MemoryRegion::new(
                        self.device.bars[usize::from(bar)].address + u64::from(offset),
                        u64::from(length),
                    );
                }

                if cfg_type == VirtioPciCapabilityType::NotifyConfig as u8 {
                    self.notify_region = mem::MemoryRegion::new(
                        self.device.bars[usize::from(bar)].address + u64::from(offset),
                        u64::from(length),
                    );

                    // struct virtio_pci_notify_cap {
                    //         struct virtio_pci_cap cap;
                    //         le32 notify_off_multiplier; /* Multiplier for queue_notify_off. */
                    // };
                    self.notify_off_multiplier = self.device.config_read_u32(cap_next + 16);
                }

                if cfg_type == VirtioPciCapabilityType::DeviceConfig as u8 {
                    self.device_config_region = mem::MemoryRegion::new(
                        self.device.bars[usize::from(bar)].address + u64::from(offset),
                        u64::from(length),
                    );
                }
            }
            cap_next = self.device.config_read_u8(cap_next + 1)
        }

        Ok(())
    }

    fn get_status(&self) -> u32 {
        // device_status: 0x14
        u32::from(self.region.io_read_u8(0x14))
    }

    fn set_status(&self, value: u32) {
        // device_status: 0x14
        self.region.io_write_u8(0x14, value as u8);
    }

    fn add_status(&self, value: u32) {
        self.set_status(self.get_status() | value);
    }

    fn reset(&self) {
        self.set_status(0);
    }

    fn get_features(&self) -> u64 {
        // device_feature_select: 0x00
        self.region.io_write_u32(0x00, 0);
        // device_feature: 0x04
        let mut device_features: u64 = u64::from(self.region.io_read_u32(0x04));
        // device_feature_select: 0x00
        self.region.io_write_u32(0x00, 1);
        // device_feature: 0x04
        device_features |= u64::from(self.region.io_read_u32(0x04)) << 32;

        device_features
    }

    fn set_features(&self, features: u64) {
        // driver_feature_select: 0x08
        self.region.io_write_u32(0x08, 0);
        // driver_feature: 0x0c
        self.region.io_write_u32(0x0c, features as u32);
        // driver_feature_select: 0x08
        self.region.io_write_u32(0x08, 1);
        // driver_feature: 0x0c
        self.region.io_write_u32(0x0c, (features >> 32) as u32);
    }

    fn set_queue(&self, queue: u16) {
        // queue_select: 0x16
        self.region.io_write_u16(0x16, queue);
    }

    fn get_queue_max_size(&self) -> u16 {
        // queue_size: 0x18
        self.region.io_read_u16(0x18)
    }

    fn set_queue_size(&self, queue_size: u16) {
        // queue_size: 0x18
        self.region.io_write_u16(0x18, queue_size);
    }

    fn set_descriptors_address(&self, addr: u64) {
        // queue_desc: 0x20
        self.region.io_write_u64(0x20, addr);
    }

    fn set_avail_ring(&self, addr: u64) {
        // queue_avail: 0x28
        self.region.io_write_u64(0x28, addr);
    }

    fn set_used_ring(&self, addr: u64) {
        // queue_used: 0x28
        self.region.io_write_u64(0x30, addr);
    }

    fn set_queue_enable(&self) {
        // queue_enable: 0x1c
        self.region.io_write_u16(0x1c, 0x1);
    }

    fn notify_queue(&self, queue: u16) {
        // queue_notify_off: 0x1e
        let queue_notify_off = self.region.io_read_u16(0x1e);

        self.notify_region.io_write_u32(
            u64::from(queue_notify_off) * u64::from(self.notify_off_multiplier),
            u32::from(queue),
        );
    }

    fn read_device_config(&self, offset: u64) -> u32 {
        self.device_config_region.io_read_u32(offset)
    }
}
