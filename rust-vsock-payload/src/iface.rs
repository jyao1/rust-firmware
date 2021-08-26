// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::virtio_vsock_device::VirtioVsockDevice;

pub struct Iface<'a> {
    device: &'a mut VirtioVsockDevice<'a>,
}

impl<'a> Iface<'a> {
    pub fn new(device: &'a mut VirtioVsockDevice<'a>) -> Iface {
        Iface { device }
    }

    pub fn start_loop(&mut self) {
        self.device.test_client();
    }
}
