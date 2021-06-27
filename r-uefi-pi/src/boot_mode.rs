// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::enum_builder;
use scroll::{ctx, Endian, Pread, Pwrite};

enum_builder! {
    /// EFI boot mode
    ///
    /// PI Version 1.2.1A
    /// 0x21 - 0xf..f are reserved.
    @U32
    EnumName: BootMode;
    EnumVal{
        BOOT_WITH_FULL_CONFIGURATION => 0x00,
        BOOT_WITH_MINIMAL_CONFIGURATION => 0x01,
        BOOT_ASSUMING_NO_CONFIGURATION_CHANGES => 0x02,
        BOOT_WITH_FULL_CONFIGURATION_PLUS_DIAGNOSTICS => 0x03,
        BOOT_WITH_DEFAULT_SETTINGS => 0x04,
        BOOT_ON_S4_RESUME => 0x05,
        BOOT_ON_S5_RESUME => 0x06,
        BOOT_WITH_MFG_MODE_SETTINGS => 0x07,
        BOOT_ON_S2_RESUME => 0x10,
        BOOT_ON_S3_RESUME => 0x11,
        BOOT_ON_FLASH_UPDATE => 0x12,
        BOOT_IN_RECOVERY_MODE => 0x20
    }
}
