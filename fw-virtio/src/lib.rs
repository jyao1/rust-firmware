// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
#![cfg_attr(not(test), no_std)]

mod dma;
mod mem;

pub mod consts;
pub mod virtio_pci;
pub mod virtqueue;

/// Virtio errors
#[derive(Debug, Eq, PartialEq)]
pub enum VirtioError {
    /// Virtio device not support.
    VirtioUnsupportedDevice,
    /// Virtio legacy device only.
    VirtioLegacyOnly,
    /// Virtio device negotiation failed.
    VirtioFeatureNegotiationFailed,
    /// VirtioQueue is too small.
    VirtioQueueTooSmall,
    /// The buffer is too small.
    BufferTooSmall,
    /// The device is not ready.
    NotReady,
    /// The queue is already in use.
    AlreadyUsed,
    /// Invalid parameter.
    InvalidParam,
    /// Failed to alloc DMA memory.
    DmaError,
    /// I/O Error
    IoError,
}

pub type Result<T = ()> = core::result::Result<T, VirtioError>;

/// Trait to allow separation of transport from block driver
pub trait VirtioTransport {
    fn init(&mut self, device_type: u32) -> Result<()>;
    fn get_status(&self) -> u32;
    fn set_status(&self, status: u32);
    fn add_status(&self, status: u32);
    fn reset(&self);
    fn get_features(&self) -> u64;
    fn set_features(&self, features: u64);
    fn set_queue(&self, queue: u16);
    fn get_queue_max_size(&self) -> u16;
    fn set_queue_size(&self, queue_size: u16);
    fn set_descriptors_address(&self, address: u64);
    fn set_avail_ring(&self, address: u64);
    fn set_used_ring(&self, address: u64);
    fn set_queue_enable(&self);
    fn notify_queue(&self, queue: u16);
    fn read_device_config(&self, offset: u64) -> u32;
}
