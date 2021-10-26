// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![feature(global_asm)]
#![cfg_attr(not(test), no_std)]

pub mod fsp_info_header;
pub mod fsp_upd_header;

mod asm;
mod memslice;
pub mod fsp;

#[cfg(test)]
mod tests {
    #[test]
    fn dump_fsp() {
        // TBD
    }
}
