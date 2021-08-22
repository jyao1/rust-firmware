// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub const VIRTIO_SUBSYSTEM_BLOCK: u32 = 2;
pub const VIRTIO_SUBSYSTEM_VSOCK: u32 = 19;

pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;

pub const VIRTIO_STATUS_RESET: u32 = 0;
pub const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
pub const VIRTIO_STATUS_DRIVER: u32 = 2;
pub const VIRTIO_STATUS_FEATURES_OK: u32 = 8;
pub const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
pub const VIRTIO_STATUS_FAILED: u32 = 128;
