// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

// QemuFspPkg FSP-S UPD

use rust_fsp_wrapper::fsp_upd_header::FspUpdHeader;
use scroll::{Pread, Pwrite};

/// Fsp S Configuration
///
#[derive(Debug, Pread, Pwrite)]
pub struct FspSConfig {
    /// Offset 0x0040 - BMP Logo Data Size
    ///   BMP logo data buffer size. 0x00000000(Default).
    pub logo_size: u32,

    /// Offset 0x0044 - BMP Logo Data Pointer
    ///   BMP logo data pointer to a BMP format buffer. 0x00000000(Default).
    pub logo_ptr: u32,

    /// Offset 0x0048 - Graphics Configuration Data Pointer
    ///   Graphics configuration data used for initialization. 0x00000000(Default).
    pub graphics_config_ptr: u32,

    /// Offset 0x004C - PCI Temporary MMIO Base
    ///   PCI Temporary MMIO Base used before full PCI enumeration. 0x80000000(Default).
    pub pci_temp_resource_base: u32,

    /// Offset 0x0050
    pub unused_upd_space1: [u8; 32],

    /// Offset 0x0070
    pub reserved_fsps_upd: u8,
}

/// Fsp S UPD Configuration
#[derive(Debug, Pread, Pwrite)]
pub struct FspsUpd {
    /// Offset 0x0000
    pub fsp_upd_header: FspUpdHeader,

    /// Offset 0x0020
    pub unused_upd_space0: [u8; 32],

    /// Offset 0x0040
    pub fsps_config: FspSConfig,

    /// Offset 0x0071
    pub unused_upd_space2: [u8; 13],

    /// Offset 0x007E
    pub upd_terminator: u16,
}
