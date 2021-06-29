// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::fmt;
use scroll::{Pread, Pwrite};

///
/// This value define FspInfoHeader offset in FSP FV
///
pub const FSP_INFO_HEADER_OFF: usize = 0x94;

#[derive(Pread, Pwrite, Default)]
pub struct FspInfoHeader {
    ///
    /// Byte 0x00: Signature ('FSPH') for the FSP Information Header.
    ///
    pub signature: u32,
    ///
    /// Byte 0x04: Length of the FSP Information Header.
    ///
    pub header_length: u32,
    ///
    /// Byte 0x08: Reserved.
    ///
    pub reserved1: [u8; 2],
    ///
    /// Byte 0x0A: Indicates compliance with a revision of this specification in the BCD format.
    ///
    pub spec_version: u8,
    ///
    /// Byte 0x0B: Revision of the FSP Information Header.
    ///
    pub header_revision: u8,
    ///
    /// Byte 0x0C: Revision of the FSP binary.
    ///
    pub image_revision: u32,
    ///
    /// Byte 0x10: Signature string that will help match the FSP Binary to a supported HW configuration.
    ///
    pub image_id: [u8; 8],
    ///
    /// Byte 0x18: Size of the entire FSP binary.
    ///
    pub image_size: u32,
    ///
    /// Byte 0x1C: FSP binary preferred base address.
    ///
    pub image_base: u32,
    ///
    /// Byte 0x20: Attribute for the FSP binary.
    ///
    pub image_attribute: u16,
    ///
    /// Byte 0x22: Attributes of the FSP Component.
    ///
    pub component_attribute: u16,
    ///
    /// Byte 0x24: Offset of the FSP configuration region.
    ///
    pub cfg_region_offset: u32,
    ///
    /// Byte 0x28: Size of the FSP configuration region.
    ///
    pub cfg_region_size: u32,
    ///
    /// Byte 0x2C: Reserved2.
    ///
    pub reserved2: u32,
    ///
    /// Byte 0x30: The offset for the API to setup a temporary stack till the memory is initialized.
    ///
    pub temp_ram_init_entry_offset: u32,
    ///
    /// Byte 0x34: Reserved3.
    ///
    pub reserved3: u32,
    ///
    /// Byte 0x38: The offset for the API to inform the FSP about the different stages in the boot process.
    ///
    pub notify_phase_entry_offset: u32,
    ///
    /// Byte 0x3C: The offset for the API to initialize the memory.
    ///
    pub fsp_memory_init_entry_offset: u32,
    ///
    /// Byte 0x40: The offset for the API to tear down temporary RAM.
    ///
    pub temp_ram_exit_entry_offset: u32,
    ///
    /// Byte 0x44: The offset for the API to initialize the CPU and chipset.
    ///
    pub fsp_silicon_init_entry_offset: u32,
    ///
    /// Byte 0x48: Offset for the API for the optional Multi-Phase processor and chipset initialization.
    ///            This value is only valid if FSP HeaderRevision is >= 5.
    ///            If the value is set to 0x00000000, then this API is not available in this component.
    ///
    pub fsp_multi_phase_si_init_entry_offset: u32,
}

impl fmt::Debug for FspInfoHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let signature = self.signature.to_ne_bytes();
        f.debug_struct("FspInfoHeader")
            .field(
                "signature",
                &format_args!(
                    "{}{}{}{}",
                    signature[0] as char,
                    signature[1] as char,
                    signature[2] as char,
                    signature[3] as char
                ),
            )
            .field("header_length", &format_args!("0x{:x}", self.header_length))
            .field("spec_version", &format_args!("0x{:x}", self.spec_version))
            .field(
                "header_revision",
                &format_args!("0x{:x}", self.header_revision),
            )
            .field(
                "image_revision",
                &format_args!("0x{:x}", self.image_revision),
            )
            .field(
                "image_id",
                &format_args!(
                    "{}{}{}{}{}{}{}{}",
                    self.image_id[0] as char,
                    self.image_id[1] as char,
                    self.image_id[2] as char,
                    self.image_id[3] as char,
                    self.image_id[4] as char,
                    self.image_id[5] as char,
                    self.image_id[6] as char,
                    self.image_id[7] as char,
                ),
            )
            .field("image_size", &format_args!("0x{:x}", self.image_size))
            .field("image_base", &format_args!("0x{:x}", self.image_base))
            .field(
                "temp_ram_init_entry_offset",
                &format_args!("0x{:x}", self.temp_ram_init_entry_offset),
            )
            .field(
                "notify_phase_entry_offset",
                &format_args!("0x{:x}", self.notify_phase_entry_offset),
            )
            .field(
                "fsp_memory_init_entry_offset",
                &format_args!("0x{:x}", self.fsp_memory_init_entry_offset),
            )
            .field(
                "temp_ram_exit_entry_offset",
                &format_args!("0x{:x}", self.temp_ram_exit_entry_offset),
            )
            .field(
                "fsp_silicon_init_entry_offset",
                &format_args!("0x{:x}", self.fsp_silicon_init_entry_offset),
            )
            .field(
                "cfg_region_offset",
                &format_args!("0x{:x}", self.cfg_region_offset),
            )
            .field(
                "cfg_region_size",
                &format_args!("0x{:x}", self.cfg_region_size),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fsp_info_header_offset() {
        use super::{FspInfoHeader, FSP_INFO_HEADER_OFF};
        use scroll::Pread;
        let fsp_info_header_bytes =
            &include_bytes!("../fsp_bins/Qemu/QEMU_FSP_RELEASE_T_FFFC5000.fd")[..];
        let fsp_info_header = fsp_info_header_bytes
            .pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF)
            .unwrap();
        assert_eq!(&fsp_info_header.signature.to_ne_bytes(), b"FSPH");
        assert_eq!(fsp_info_header.temp_ram_init_entry_offset, 0x593);
    }
}
