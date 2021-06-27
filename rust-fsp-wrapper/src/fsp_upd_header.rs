// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use scroll::{Pread, Pwrite};

///
/// FSP_UPD_HEADER Configuration.
///
#[derive(Debug, Pread, Pwrite, Default)]
pub struct FspUpdHeader {
    ///
    /// UPD Region Signature. This signature will be
    /// "XXXXXX_T" for FSP-T
    /// "XXXXXX_M" for FSP-M
    /// "XXXXXX_S" for FSP-S
    /// Where XXXXXX is an unique signature
    ///
    pub signature: u64,
    ///
    /// Revision of the Data structure.
    ///   For FSP spec 2.0/2.1 value is 1.
    ///   For FSP spec 2.2 value is 2.
    ///
    pub revision: u8,
    pub reserved: [u8; 23],
}

///
/// FSPM_ARCH_UPD Configuration.
///
#[derive(Debug, Pread, Pwrite, Default)]
pub struct FspmArchUpd {
    ///
    /// Revision of the structure. For FSP v2.0 value is 1.
    ///
    pub revision: u8,
    pub reserved: [u8; 3],
    ///
    /// Pointer to the non-volatile storage (NVS) data buffer.
    /// If it is NULL it indicates the NVS data is not available.
    ///
    pub nvs_buffer_ptr: u32,
    ///
    /// Pointer to the temporary stack base address to be
    /// consumed inside FspMemoryInit() API.
    ///
    pub stack_base: u32,
    ///
    /// Temporary stack size to be consumed inside
    /// FspMemoryInit() API.
    ///
    pub stack_size: u32,
    ///
    /// Size of memory to be reserved by FSP below "top
    /// of low usable memory" for bootloader usage.
    ///
    pub boot_loader_tolum_size: u32,
    ///
    /// Current boot mode.
    ///
    pub boot_mode: u32,
    ///
    /// Optional event handler for the bootloader to be informed of events occurring during FSP execution.
    /// This value is only valid if Revision is >= 2.
    ///
    pub fsp_event_handler: u32,
    pub reserved1: [u8; 4],
}
