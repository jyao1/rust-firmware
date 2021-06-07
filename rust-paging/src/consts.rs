// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub const PHYS_VIRT_OFFSET: usize = 0;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_BASE: u64 = 0x810000 + 0x020000; // pcd::pcd_get_PcdOvmfSecPeiTempRamBase() + pcd::pcd_get_PcdOvmfSecPeiTempRamSize()
pub const PAGE_TABLE_SIZE: usize = 0x100000;