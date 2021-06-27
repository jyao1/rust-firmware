// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::enum_builder;
use scroll::{ctx, Endian, Pread, Pwrite};

enum_builder! {
    /// HobType of EFI_HOB_GENERIC_HEADER.
    ///
    /// PI Version 1.6
    @U16
    EnumName: HobType;
    EnumVal{
        HANDOFF => 0x0001,
        MEMORY_ALLOCATION => 0x0002,
        RESOURCE_DESCRIPTOR => 0x0003,
        GUID_EXTENSION => 0x0004,
        FV => 0x0005,
        CPU => 0x0006,
        MEMORY_POOL => 0x0007,
        FV2 => 0x0009,
        LOAD_PEIM_UNUSED => 0x000A,
        UEFI_CAPSULE => 0x000B,
        FV3 => 0x000C,
        UNUSED => 0xfffe,
        END_OF_HOB_LIST => 0xffff
    }
}
