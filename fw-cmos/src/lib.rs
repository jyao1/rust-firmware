// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![feature(core_intrinsics)]
#![cfg_attr(not(test),no_std)]

mod cmos;
pub use cmos::*;
