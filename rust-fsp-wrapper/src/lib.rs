// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]

pub mod fsp_info_header;
pub mod fsp_m_udp;
pub mod fsp_s_udp;
pub mod fsp_t_upd;
pub mod fsp_upd_header;

#[cfg(test)]
mod tests {
    #[test]
    fn dump_fsp() {}
}
