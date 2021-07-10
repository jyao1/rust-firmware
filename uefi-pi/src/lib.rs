// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub mod fv_lib;
pub mod hob_lib;
pub mod const_guids;
pub mod pi {
    pub use crate::fv_lib;
    pub use crate::hob_lib;
}
