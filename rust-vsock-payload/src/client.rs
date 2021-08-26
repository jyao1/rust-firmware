// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use fw_vsock::vsock::{VsockAddr, VsockStream};

#[allow(dead_code)]
pub fn test_client() {
    log::debug!("test client\n");
    let mut s = VsockStream::new();
    s.connect(&VsockAddr::new(2, 1234)).expect("error");
    log::debug!("connected \n");
    let nsend = s.send(b"hello", 0).unwrap();
    log::debug!("send {} bytes!\n", nsend);
}
