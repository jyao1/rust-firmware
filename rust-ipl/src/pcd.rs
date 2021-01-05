// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfSecPeiTempRamSize() -> u32 {
    0x020000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfSecPeiTempRamBase() -> u32 {
    0x810000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfSecPageTablesBase() -> u32 {
    0x800000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfDxeMemFvBase() -> u32 {
    0xFFC84000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfDxeMemFvSize() -> u32 {
    0x00248000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfPeiMemFvBase() -> u32 {
    0xFFECC000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdOvmfPeiMemFvSize() -> u32 {
    0x00134000
}

#[allow(non_snake_case)]
pub fn pcd_get_PcdPciExpressBaseAddress() -> u64 {
    0x80000000
}
