// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::enum_builder;
use scroll::{ctx, Endian, Pread, Pwrite};

enum_builder! {
    ///
    /// The EFI_HOB_RESOURCE_DESCRIPTOR.
    ///
    @U32
    EnumName: ResourceType;
    EnumVal{
        SYSTEM_MEMORY => 0x00,
        MEMORY_MAPPED_IO => 0x01,
        IO => 0x02,
        FIRMWARE_DEVICE => 0x03,
        MEMORY_MAPPED_IO_PORT => 0x04,
        MEMORY_RESERVED => 0x05,
        IO_RESERVED => 0x06
    }
}
