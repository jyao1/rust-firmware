// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use rust_fsp_wrapper::fsp_t_upd::{FsptUpd, FsptCommonUpd};
use rust_fsp_wrapper::fsp_upd_header::FspUpdHeader;


const FSPT_UPD_SIGNATURE : u64 = 0x545F4450554D4551;        /* 'QEMUPD_T' */
// const FSPM_UPD_SIGNATURE : u64 = 0x4D5F4450554D4551;        /* 'QEMUPD_M' */
// const FSPS_UPD_SIGNATURE : u64 = 0x535F4450554D4551;        /* 'QEMUPD_S' */


pub const TEMP_RAM_INIT_PARAM: FsptUpd = FsptUpd {
    fsp_upd_header: FspUpdHeader {
        signature: FSPT_UPD_SIGNATURE,
        revision: 1,
        reserved: [0u8; 23],
    },
    fspt_common_upd: FsptCommonUpd {
        revision: 1,
        reserved: [0u8; 3],
        microcode_region_base: 0,
        microcode_region_length: 0,
        code_region_base: 0,
        code_region_length: 0,
        reserved1: [0u8; 12],
    },
    reserved_fspt_upd1: [0u8; 32],
    unused_upd_space0: [0u8; 48],
    upd_terminator: 0x55AA,
};
