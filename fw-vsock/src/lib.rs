// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]

#[derive(Debug)]
pub enum VsockError {
    /// Device io error
    DeviceError,
    /// Packet buffer is too short.
    Truncated,
    /// Packet header can not be recognized.
    Malformed,

    /// VsockStream
    /// An operation is not permitted in the current state.
    Illegal,
    /// There is no listen socket on remote
    REFUSED,
}

pub type Result<T = ()> = core::result::Result<T, VsockError>;

pub mod device;
pub mod protocol;
pub mod virtio_vsock_device;
pub mod vsock;
