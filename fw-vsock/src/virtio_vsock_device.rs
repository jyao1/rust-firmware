// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::{device, VsockError};

use atomic_refcell::AtomicRefCell as RefCell;
use fw_virtio::{consts::*, virtqueue::VirtQueue, Result, VirtioError, VirtioTransport};

const QUEUE_SIZE: usize = 4;

pub const QUEUE_RX: u16 = 0;
pub const QUEUE_TX: u16 = 1;
pub const QUEUE_EVENT: u16 = 2;

pub const MAX_VSOCK_MTU: usize = 1024;

#[repr(C)]
#[repr(align(64))]
/// Device driver for virtio block over any transport
pub struct VirtioVsockDevice<T>
where
    T: VirtioTransport,
{
    transport: T,
    rx: RefCell<VirtQueue>,
    tx: RefCell<VirtQueue>,
    event: RefCell<VirtQueue>,
}

impl<T> VirtioVsockDevice<T>
where
    T: VirtioTransport,
{
    pub fn new(transport: T) -> Result<VirtioVsockDevice<T>> {
        // Initialise the transport
        let mut transport = transport;
        transport.init(VIRTIO_SUBSYSTEM_VSOCK)?;

        // Reset device
        transport.add_status(64);
        transport.set_status(VIRTIO_STATUS_RESET);

        // Acknowledge
        transport.add_status(VIRTIO_STATUS_ACKNOWLEDGE);

        // And advertise driver
        transport.add_status(VIRTIO_STATUS_DRIVER);

        // And device features ok
        transport.add_status(VIRTIO_STATUS_FEATURES_OK);
        if transport.get_status() & VIRTIO_STATUS_FEATURES_OK != VIRTIO_STATUS_FEATURES_OK {
            transport.add_status(VIRTIO_STATUS_FAILED);
            log::info!("VirtioFeatureNegotiationFailed");
            return Err(VirtioError::VirtioFeatureNegotiationFailed);
        }

        // Hardcoded queue size to QUEUE_SIZE at the moment
        let max_queue = transport.get_queue_max_size();
        if max_queue < QUEUE_SIZE as u16 {
            log::info!("max_queue: {}\n", max_queue);
            transport.add_status(VIRTIO_STATUS_FAILED);
            return Err(VirtioError::VirtioQueueTooSmall);
        }
        transport.set_queue_size(QUEUE_SIZE as u16);

        // program queue rx(idx 0)
        let queue_rx = Self::create_queue(&transport, QUEUE_RX, QUEUE_SIZE as u16)?;

        // program queues tx(idx 1)
        let queue_tx = Self::create_queue(&transport, QUEUE_TX, QUEUE_SIZE as u16)?;

        // program queues event(idx 2)
        let queue_event = Self::create_queue(&transport, QUEUE_EVENT, QUEUE_SIZE as u16)?;

        Ok(VirtioVsockDevice {
            transport,
            rx: RefCell::new(queue_rx),
            tx: RefCell::new(queue_tx),
            event: RefCell::new(queue_event),
        })
    }

    pub fn init(&self) -> Result {
        // Report driver ready
        self.transport.add_status(VIRTIO_STATUS_DRIVER_OK);

        if self.transport.get_status() & VIRTIO_STATUS_DRIVER_OK != VIRTIO_STATUS_DRIVER_OK {
            self.transport.add_status(VIRTIO_STATUS_FAILED);
            log::info!("VIRTIO_STATUS_DRIVER_OK failed");
            return Err(VirtioError::VirtioFeatureNegotiationFailed);
        }
        log::info!("VIRTIO_STATUS_DRIVER_OK set\n");

        Ok(())
    }

    // Get current device CID
    pub fn get_cid(&self) -> u64 {
        u64::from(self.transport.read_device_config(0))
            | u64::from(self.transport.read_device_config(4)) << 32
    }

    /// Whether can send packet.
    pub fn can_send(&self) -> bool {
        let tx = self.tx.borrow();
        tx.available_desc() >= 1
    }

    /// Whether can receive packet.
    pub fn can_recv(&self) -> bool {
        let rx = self.rx.borrow();
        rx.can_pop()
    }

    /// Receive a packet.
    pub fn recv(&self, bufs: &[&mut [u8]]) -> Result<usize> {
        let mut rx = self.rx.borrow_mut();
        rx.add(&[], bufs)?;

        self.transport.set_queue(QUEUE_RX);
        self.transport.notify_queue(QUEUE_RX);

        while !rx.can_pop() {}

        let (_, len) = rx.pop_used()?;
        Ok(len as usize)
    }

    /// Send a packet
    pub fn send(&self, bufs: &[&[u8]]) -> Result<usize> {
        let mut tx = self.tx.borrow_mut();

        tx.add(bufs, &[])?;

        self.transport.set_queue(QUEUE_TX);
        self.transport.notify_queue(QUEUE_TX);

        while !tx.can_pop() {}

        let (_, len) = tx.pop_used()?;

        Ok(len as usize)
        // Ok(0)
    }

    /// create and enable a virtqueue and enable it.
    fn create_queue(
        transport: &dyn VirtioTransport,
        idx: u16,
        queue_size: u16,
    ) -> Result<VirtQueue> {
        transport.set_queue(idx);
        transport.set_queue_size(queue_size);
        let queue = VirtQueue::new(transport, idx as usize, queue_size)?;
        transport.set_queue_enable();
        Ok(queue)
    }
}

pub struct RxToken {
    buffer: [u8; MAX_VSOCK_MTU],
    length: usize,
}

impl Default for RxToken {
    fn default() -> Self {
        Self {
            buffer: [0u8; MAX_VSOCK_MTU],
            length: MAX_VSOCK_MTU,
        }
    }
}

impl RxToken {
    pub fn new() -> Self {
        RxToken::default()
    }
}

impl AsMut<[u8]> for RxToken {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[0..self.length]
    }
}

impl AsRef<[u8]> for RxToken {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }
}

impl device::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> crate::Result<R>
    where
        F: FnOnce(&mut [u8]) -> crate::Result<R>,
    {
        f(&mut self.buffer[..self.length])
    }
}

pub struct TxToken<'a, T: VirtioTransport> {
    lower: &'a mut VirtioVsockDevice<T>,
}

impl<'a, T> device::TxToken for TxToken<'a, T>
where
    T: VirtioTransport,
{
    fn consume<R, F>(self, len: usize, f: F) -> crate::Result<R>
    where
        F: FnOnce(&mut [u8]) -> crate::Result<R>,
    {
        let lower = self.lower;
        let mut buffer = [0; MAX_VSOCK_MTU];
        let result = f(&mut buffer[..len]);
        let _ = lower
            .send(&[&buffer[..len]])
            .map_err(|_| VsockError::DeviceError)?;
        result
    }
}

impl<'a, T: 'a> device::Device<'a> for VirtioVsockDevice<T>
where
    T: VirtioTransport,
{
    type RxToken = RxToken;

    type TxToken = TxToken<'a, T>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        log::info!("enter recv\n");
        let mut rx = RxToken::new();
        match self.recv(&[rx.as_mut()]) {
            Ok(len) => {
                rx.length = len;
                let tx = TxToken { lower: self };
                Some((rx, tx))
            }
            Err(_) => None,
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(TxToken { lower: self })
    }
}
