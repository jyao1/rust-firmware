// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

// QemuFspPkg FSP-S UPD

use rust_fsp_wrapper::fsp_upd_header::{FspUpdHeader, FspmArchUpd};
use scroll::{Pread, Pwrite};

///
/// Fsp M Configuration
///
#[derive(Debug, Pread, Pwrite)]
pub struct FspmConfig {
    ///
    /// Offset 0x0040 - Debug Serial Port Base address
    ///   Debug serial port base address. This option will be used only when the 'Serial Port
    ///   Debug Device' option is set to 'External Device'. 0x00000000(Default).
    ///
    pub serial_debug_port_address: u32,
    ///
    /// Offset 0x0044 - Debug Serial Port Type
    ///   16550 compatible debug serial port resource type. NONE means no serial port support.
    ///   0x02:MMIO(Default).
    ///   0:NONE, 1:I/O, 2:MMIO
    ///
    pub serial_debug_port_type: u8,
    ///
    /// Offset 0x0045 - Serial Port Debug Device
    ///   Select active serial port device for debug. For SOC UART devices,'Debug Serial Port
    ///   Base' options will be ignored. 0x02:SOC UART2(Default).
    ///   0:SOC UART0, 1:SOC UART1, 2:SOC UART2, 3:External Device
    ///
    pub serial_debug_port_device: u8,
    ///
    /// Offset 0x0046 - Debug Serial Port Stride Size
    ///   Debug serial port register map stride size in bytes. 0x00:1, 0x02:4(Default).
    ///   0:1, 2:4
    ///
    pub serial_debug_port_stride_size: u8,
    ///
    /// Offset 0x0047
    ///
    pub unused_upd_space0: [u8; 49],
    ///
    /// Offset 0x0078
    ///
    pub reserved_fspm_upd: [u8; 4],
}

///
/// Fsp M UPD Configuration
///
#[derive(Debug, Pread, Pwrite, Default)]
pub struct FspmUpd {
    ///
    /// Offset 0x0000
    ///
    pub fsp_upd_header: FspUpdHeader,
    ///
    /// Offset 0x0020
    ///
    pub fspm_arch_upd: FspmArchUpd,
    ///
    /// Offset 0x0040
    ///
    pub fspm_config: FspmConfig,
    ///
    /// Offset 0x007C
    ///
    pub unused_upd_space1: [u8; 2],
    ///
    /// Offset 0x007E
    ///
    pub upd_terminator: u16,
}

impl Default for FspmConfig {
    fn default() -> Self {
        FspmConfig {
            serial_debug_port_address: 0,
            serial_debug_port_type: 0,
            serial_debug_port_device: 0,
            serial_debug_port_stride_size: 0,
            unused_upd_space0: [0u8; 49],
            reserved_fspm_upd: [0u8; 4],
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fsp_info_header_default_memory_udp() {
        use super::FspmUpd;
        use rust_fsp_wrapper::fsp_info_header::{FspInfoHeader, FSP_INFO_HEADER_OFF};
        use scroll::Pread;
        let fsp_info_header_bytes =
            &include_bytes!("../../../rust-fsp-wrapper/fsp_bins/Qemu/QEMU_FSP_RELEASE_M_FFFC8000.fd")[..];
        let fsp_info_header = fsp_info_header_bytes
            .pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF)
            .unwrap();
        assert_eq!(&fsp_info_header.signature.to_ne_bytes(), b"FSPH");
        let cfg_range = fsp_info_header.cfg_region_offset as usize
            ..(fsp_info_header.cfg_region_offset + fsp_info_header.cfg_region_size) as usize;

        let fspm_upd = &fsp_info_header_bytes[cfg_range]
            .pread::<FspmUpd>(0)
            .unwrap();
        println!("{:x?}", fspm_upd);
    }
}
