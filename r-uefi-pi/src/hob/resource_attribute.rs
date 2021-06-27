// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use bitflags::bitflags;
use scroll::{Pread, Pwrite};

bitflags! {
    #[derive(Default, Pread, Pwrite)]
    pub struct ResourceAttributeType: u32 {
        const PRESENT = 0x00000001;
        const INITIALIZED = 0x00000002;
        const TESTED = 0x00000004;
        const READ_PROTECTED = 0x00000080;
        const WRITE_PROTECTED = 0x00000100;
        const EXECUTION_PROTECTED = 0x00000200;
        const PERSISTENT = 0x00800000;
        const SINGLE_BIT_ECC = 0x00000008;
        const MULTIPLE_BIT_ECC = 0x00000010;
        const ECC_RESERVED_1 = 0x00000020;
        const ECC_RESERVED_2 = 0x00000040;
        const UNCACHEABLE = 0x00000400;
        const WRITE_COMBINEABLE = 0x00000800;
        const WRITE_THROUGH_CACHEABLE = 0x00001000;
        const WRITE_BACK_CACHEABLE = 0x00002000;
        const BIT16_IO = 0x00004000;
        const BIT32_IO = 0x00008000;
        const BIT64_IO = 0x00010000;
        const UNCACHED_EXPORTED = 0x00020000;
        const READ_PROTECTABLE = 0x00100000;
        const WRITE_PROTECTABLE = 0x00200000;
        const EXECUTION_PROTECTABLE = 0x00400000;
        const PERSISTABLE = 0x01000000;
        const READ_ONLY_PROTECTED = 0x00040000;
        const READ_ONLY_PROTECTABLE = 0x00080000;
        const MORE_RELIABLE = 0x02000000;
    }
}
