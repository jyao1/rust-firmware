// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

///
///  QemuFspPkg
///

mod fsp_t_upd;
mod fsp_m_upd;
mod fsp_s_upd;

mod fsp_data;

pub use fsp_t_upd::FsptUpd;
pub use fsp_m_upd::{FspmUpd, FspmConfig};
pub use fsp_s_upd::{FspsUpd, FspSConfig};
pub use fsp_data::{TEMP_RAM_INIT_PARAM, FSPT_UPD_SIGNATURE, FSPM_UPD_SIGNATURE, FSPS_UPD_SIGNATURE};
