// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]

#[cfg(feature="qemu")]
mod qemu;
#[cfg(feature="qemu")]
pub use qemu::*;
