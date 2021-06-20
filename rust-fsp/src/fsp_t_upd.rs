// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::default::Default;

use scroll::{Pread, Pwrite};

use crate::fsp_upd_header::FspUpdHeader;

#[derive(Debug, Pread, Pwrite, Default)]
pub struct FsptCommonUpd {
    pub revision: u8,
    pub reserved: [u8; 3],
    pub microcode_region_base: u32,
    pub microcode_region_length: u32,
    pub code_region_base: u32,
    pub code_region_length: u32,
    pub reserved1: [u8; 12],
}

#[derive(Debug, Pread, Pwrite)]
pub struct FsptUpd {
    pub fsp_upd_header: FspUpdHeader,
    pub fspt_common_upd: FsptCommonUpd,
    pub reserved_fspt_upd1: [u8; 32],
    pub unused_upd_space0: [u8; 48],
    pub upd_terminator: u16,
}

impl Default for FsptUpd {
    fn default() -> Self {
        FsptUpd {
            fsp_upd_header: FspUpdHeader::default(),
            fspt_common_upd: FsptCommonUpd::default(),
            reserved_fspt_upd1: [0u8; 32],
            unused_upd_space0: [0u8; 48],
            upd_terminator: 0u16,
        }
    }
}
