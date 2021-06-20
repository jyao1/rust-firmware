// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use scroll::{Pread, Pwrite};

#[derive(Debug, Pread, Pwrite, Default)]
pub struct FspUpdHeader {
    ///
    /// UPD Region Signature. This signature will be
    /// "XXXXXX_T" for FSP-T
    /// "XXXXXX_M" for FSP-M
    /// "XXXXXX_S" for FSP-S
    /// Where XXXXXX is an unique signature
    ///
    pub signature:  u64,
    pub revision:   u8,
    pub reserved: [u8; 23],
}
