// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc::vec::Vec;
use fw_virtio::virtio_pci::VirtioPciTransport;
use fw_virtio::VirtioTransport;
use fw_vsock::device::{Device, RxToken, TxToken};

#[cfg(not(test))]
mod heap;

mod platform;
mod virtio_impl;
mod vsock_impl;

mod client;
mod server;

use fw_vsock::protocol::Packet as VsockPacket;

#[no_mangle]
#[cfg_attr(target_os = "uefi", export_name = "efi_main")]
pub extern "win64" fn _start(hob_list: *const u8, _reserved_param: usize) -> ! {
    rust_ipl_log::write_log(
        rust_ipl_log::LOG_LEVEL_INFO,
        rust_ipl_log::LOG_MASK_COMMON,
        format_args!("Enter rust vsock payload\n"),
    );
    rust_ipl_log::init_with_level(log::Level::Trace);
    log::debug!("Logger init\n");
    fw_exception::setup_exception_handlers();

    let hob_list = unsafe_get_hob_from_ipl(hob_list);
    uefi_pi::hob_lib::dump_hob(hob_list);
    #[cfg(not(test))]
    if !heap::init_heap(hob_list) {
        panic!("heap init failed\n");
    }
    fw_pci::print_bus();

    platform::init();
    let dma = Vec::<u8>::with_capacity(1024 * 1024);
    virtio_impl::init(dma.as_ptr() as usize, dma.capacity());

    vsock_impl::init_vsock_device();

    // client::test_client();
    server::test_server();
    // _start_example();

    log::debug!("Example done\n");
    loop {}
}

#[cfg(target_os = "uefi")]
use core::panic::PanicInfo;

#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rust_ipl_log::write_log(
        rust_ipl_log::LOG_LEVEL_ERROR,
        rust_ipl_log::LOG_MASK_COMMON,
        format_args!("panic ... {:?}\n", _info),
    );
    loop {}
}

fn unsafe_get_hob_from_ipl<'a>(hob: *const u8) -> &'a [u8] {
    const SIZE_4M: usize = 0x40_0000;
    let hob = unsafe { core::slice::from_raw_parts(hob as *const u8, SIZE_4M) };
    let hob_size = uefi_pi::hob_lib::get_hob_total_size(hob).expect("Get hob size failed");
    &hob[..hob_size] as _
}

fn _start_example() {
    let device = vsock_impl::get_vsock_device_mut();
    _use_device_direct::<VirtioPciTransport>(device);
    log::info!("start example end");
}

fn _use_device_direct<'a, T: VirtioTransport>(
    virtio_vsock_device: &'a mut fw_vsock::virtio_vsock_device::VirtioVsockDevice<
        'a,
        VirtioPciTransport,
    >,
) {
    let (rx, tx) = virtio_vsock_device.receive().unwrap();

    // get request
    let _ = rx
        .consume(|recv_buffer| {
            log::info!("recv_buffer: {:02x?}\n", recv_buffer);
            let packet_request = VsockPacket::new_checked(recv_buffer)?;
            // send response
            tx.consume(44, |buffer| {
                let mut packet = VsockPacket::new_unchecked(buffer);
                packet.set_src_cid(33);
                packet.set_dst_cid(packet_request.src_cid());
                packet.set_src_port(1234);
                packet.set_dst_port(packet_request.src_port());
                packet.set_type(fw_vsock::protocol::field::TYPE_STREAM);
                packet.set_op(fw_vsock::protocol::field::OP_RESPONSE);
                packet.set_data_len(0);
                packet.set_flags(0);
                packet.set_fwd_cnt(0);
                packet.set_buf_alloc(262144);
                Ok(())
            })
            //
        })
        .expect("recv_error");
}
