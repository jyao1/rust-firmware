// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::protocol::field::{FLAG_SHUTDOWN_READ, FLAG_SHUTDOWN_WRITE, HEADER_LEN};
use crate::protocol::{field, Packet};
use crate::virtio_vsock_device::{VirtioVsockDevice, MAX_VSOCK_MTU};
use crate::{Result, VsockError};
use core::fmt;
use fw_virtio::virtio_pci::VirtioPciTransport;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct VsockAddr {
    cid: u64,
    port: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Closed,
    Listen,
    RequestSend,
    Establised,
    Closing,
}

pub struct VsockStream {
    state: State,
    listen_addr: VsockAddr,
    listen_backlog: u32,
    local_addr: VsockAddr,
    remote_addr: VsockAddr,

    rx_buffer: [u8; MAX_VSOCK_MTU],
    rx_cnt: u32,
}

impl Default for VsockStream {
    fn default() -> Self {
        let mut local_addr = VsockAddr::default();
        let local_cid = get_vsock_device().get_cid() as u32;
        log::debug!("get local_cid: {}\n", local_cid);
        local_addr.set_cid(local_cid);
        VsockStream {
            rx_buffer: [0u8; MAX_VSOCK_MTU],
            local_addr: local_addr,
            state: State::default(),
            listen_addr: VsockAddr::default(),
            listen_backlog: 0,
            remote_addr: VsockAddr::default(),
            rx_cnt: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Closed
    }
}

impl VsockStream {
    pub fn new() -> Self {
        VsockStream::default()
    }

    pub fn bind(&mut self, addr: &VsockAddr) -> Result {
        self.listen_addr = *addr;
        Ok(())
    }

    pub fn listen(&mut self, backlog: u32) -> Result {
        if self.state == State::Closed {
            self.listen_backlog = backlog;
            self.local_addr.set_port(self.listen_addr.port);
            self.state = State::Listen;

            // loop until new packet recieved
            // TBD: need support more
            loop {
                let vsock_device = get_vsock_device();
                let nrecv = vsock_device
                    .recv(&[&mut self.rx_buffer[..]])
                    .map_err(|_| VsockError::DeviceError)?;
                let packet = Packet::new_checked(&self.rx_buffer[..nrecv])?;
                if packet.op() == field::OP_REQUEST {
                    return Ok(());
                }
                // drop
            }
        } else {
            Err(VsockError::Illegal)
        }
    }

    pub fn accept(&self) -> Result<(VsockStream, VsockAddr)> {
        if self.state != State::Listen {
            return Err(VsockError::Illegal);
        }
        let packet_request = Packet::new_checked(&self.rx_buffer[..field::HEADER_LEN])?;
        let mut packet_buf = [0u8; field::HEADER_LEN];
        let mut packet = Packet::new_unchecked(&mut packet_buf[..]);
        packet.set_src_cid(self.local_addr.cid() as u64);
        packet.set_dst_cid(packet_request.src_cid());
        packet.set_src_port(self.local_addr.port());
        packet.set_dst_port(packet_request.src_port());
        packet.set_type(field::TYPE_STREAM);
        packet.set_op(field::OP_RESPONSE);
        packet.set_data_len(0);
        packet.set_flags(0);
        packet.set_fwd_cnt(0);
        packet.set_buf_alloc(MAX_VSOCK_MTU as u32);

        let sendn = get_vsock_device()
            .send(&[packet.as_ref()])
            .map_err(|_| VsockError::DeviceError)?;

        if sendn == field::HEADER_LEN {
            let peer_addr =
                VsockAddr::new(packet_request.src_cid() as u32, packet_request.src_port());
            Ok((
                VsockStream {
                    state: State::Establised,
                    listen_addr: VsockAddr::default(),
                    listen_backlog: 0,
                    local_addr: self.local_addr,
                    remote_addr: peer_addr,
                    rx_buffer: [0u8; MAX_VSOCK_MTU],
                    rx_cnt: 0,
                },
                peer_addr,
            ))
        } else {
            Err(VsockError::Illegal)
        }
    }

    pub fn connect(&mut self, addr: &VsockAddr) -> Result {
        log::debug!("start connecting...\n");
        if self.state != State::Closed {
            return Err(VsockError::Illegal);
        }
        self.remote_addr = *addr;
        self.local_addr.set_port(get_unused_port());
        let mut buf = [0; HEADER_LEN];
        let mut packet = Packet::new_unchecked(&mut buf[..]);
        packet.set_src_cid(self.local_addr.cid() as u64);
        packet.set_dst_cid(self.remote_addr.cid() as u64);
        packet.set_src_port(self.local_addr.port());
        packet.set_dst_port(self.remote_addr.port());
        packet.set_type(field::TYPE_STREAM);
        packet.set_op(field::OP_REQUEST);
        packet.set_data_len(0);
        packet.set_flags(0);
        packet.set_fwd_cnt(0);
        packet.set_buf_alloc(MAX_VSOCK_MTU as u32);

        let nsend = get_vsock_device()
            .send(&[packet.as_ref()])
            .map_err(|_| VsockError::DeviceError)?;
        log::debug!("send buffer {:02x?}\n", packet.as_ref());
        if nsend == HEADER_LEN {
            self.state = State::RequestSend;
        } else {
            return Err(VsockError::DeviceError);
        }

        let mut recv_buf = [0u8; HEADER_LEN];
        let nread = get_vsock_device()
            .recv(&[&mut recv_buf[..]])
            .map_err(|_| VsockError::DeviceError)?;
        let packet = Packet::new_checked(&recv_buf[..nread])?;
        if packet.r#type() == field::TYPE_STREAM
            && packet.dst_cid() == self.local_addr.cid() as u64
            && packet.dst_port() == self.local_addr.port()
            && packet.op() == field::OP_RESPONSE
            && packet.src_port() == self.remote_addr.port()
            && packet.src_cid() == self.remote_addr.cid() as u64
        {
            self.state = State::Establised;
            log::debug!("connected {}\n", self.remote_addr);
            Ok(())
        } else {
            Err(VsockError::REFUSED)
        }
    }

    pub fn shutdown(&mut self) -> Result {
        if self.state == State::Establised {
            let mut buf = [0; HEADER_LEN];
            let mut packet = Packet::new_unchecked(&mut buf[..]);
            packet.set_src_cid(self.local_addr.cid() as u64);
            packet.set_dst_cid(self.remote_addr.cid() as u64);
            packet.set_src_port(self.local_addr.port());
            packet.set_dst_port(self.remote_addr.port());
            packet.set_type(field::TYPE_STREAM);
            packet.set_op(field::OP_SHUTDOWN);
            packet.set_data_len(0);
            packet.set_flags(FLAG_SHUTDOWN_READ | FLAG_SHUTDOWN_WRITE);
            packet.set_fwd_cnt(self.rx_cnt);
            packet.set_buf_alloc(MAX_VSOCK_MTU as u32);
            let _ = get_vsock_device()
                .send(&[packet.as_ref()])
                .map_err(|_| VsockError::DeviceError);
            self.state = State::Closing;
            self.reset()
        } else {
            Err(VsockError::Illegal)
        }
    }

    pub fn send(&mut self, buf: &[u8], _flags: u32) -> Result<usize> {
        let state = self.state;
        if state != State::Establised {
            return Err(VsockError::Illegal);
        }
        let mut header_buf = [0u8; HEADER_LEN];
        let mut packet = Packet::new_unchecked(&mut header_buf[..]);
        packet.set_src_cid(self.local_addr.cid() as u64);
        packet.set_dst_cid(self.remote_addr.cid() as u64);
        packet.set_src_port(self.local_addr.port());
        packet.set_dst_port(self.remote_addr.port());
        packet.set_type(field::TYPE_STREAM);
        packet.set_op(field::OP_RW);
        packet.set_data_len(buf.len() as u32);
        packet.set_flags(0);
        packet.set_fwd_cnt(self.rx_cnt);
        packet.set_buf_alloc(MAX_VSOCK_MTU as u32);

        let nsend = get_vsock_device()
            .send(&[packet.as_ref(), buf])
            .map_err(|_| VsockError::DeviceError)?;
        Ok(nsend)
    }

    pub fn recv(&mut self, buf: &mut [u8], _flags: u32) -> Result<usize> {
        let state = self.state;
        if state != State::Establised {
            return Err(VsockError::Illegal);
        }

        let mut buf_header = [0u8; HEADER_LEN];
        let _nrecv = get_vsock_device()
            .recv(&[&mut buf_header[..], buf])
            .map_err(|_| VsockError::DeviceError)?;

        let packet = Packet::new_unchecked(&buf_header[..]);

        if packet.op() == field::OP_SHUTDOWN {
            self.shutdown()?;
            return Ok(0);
        }
        if packet.op() == field::OP_RST {
            self.reset()?;
            return Err(VsockError::Illegal);
        }

        self.rx_cnt += packet.data_len() as u32;
        Ok(packet.data_len() as usize)
    }

    fn reset(&mut self) -> Result {
        log::debug!("start reset...\n");
        if self.state == State::Closing {
            let mut buf = [0; HEADER_LEN];
            let _ = get_vsock_device()
                .recv(&[&mut buf[..]])
                .map_err(|_| VsockError::DeviceError)?;
            let packet = Packet::new_checked(&buf[..])?;
            if packet.op() == field::OP_RST {
                let mut buf = [0; HEADER_LEN];
                let mut packet = Packet::new_unchecked(&mut buf[..]);
                packet.set_src_cid(self.local_addr.cid() as u64);
                packet.set_dst_cid(self.remote_addr.cid() as u64);
                packet.set_src_port(self.local_addr.port());
                packet.set_dst_port(self.remote_addr.port());
                packet.set_type(field::TYPE_STREAM);
                packet.set_op(field::OP_RST);
                packet.set_data_len(0);
                packet.set_flags(0);
                packet.set_fwd_cnt(self.rx_cnt);
                packet.set_buf_alloc(MAX_VSOCK_MTU as u32);

                let _ = get_vsock_device().send(&[packet.as_ref()]);
                self.state = State::Closed;
                Ok(())
            } else {
                self.state = State::Closing;
                Ok(())
            }
        } else {
            Err(VsockError::Illegal)
        }
    }
}

impl Drop for VsockStream {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

impl VsockAddr {
    pub fn new(cid: u32, port: u32) -> Self {
        VsockAddr {
            cid: cid as u64,
            port,
        }
    }

    pub fn cid(&self) -> u32 {
        self.cid as u32
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn set_cid(&mut self, cid: u32) {
        self.cid = cid as u64;
    }

    pub fn set_port(&mut self, port: u32) {
        self.port = port;
    }
}

impl fmt::Display for VsockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cid: {} port: {}", self.cid(), self.port())
    }
}

impl fmt::Debug for VsockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

extern "C" {
    fn get_vsock_device_call() -> u64;
}

#[allow(unused)]
fn get_vsock_device() -> &'static VirtioVsockDevice<VirtioPciTransport> {
    unsafe {
        let res = get_vsock_device_call() as *const core::ffi::c_void
            as *const VirtioVsockDevice<VirtioPciTransport>;
        &*res
    }
}

#[allow(unused)]
fn get_vsock_device_mut() -> &'static mut VirtioVsockDevice<VirtioPciTransport> {
    unsafe {
        let res = get_vsock_device_call() as *const core::ffi::c_void
            as *mut VirtioVsockDevice<VirtioPciTransport>;
        &mut *res
    }
}

/// TBD:
pub fn get_unused_port() -> u32 {
    40000
}
