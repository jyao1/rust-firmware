// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc::vec::Vec;

#[cfg(not(test))]
mod heap;

mod platform;
mod virtio_impl;
mod vsock_impl;

mod client;
mod server;
mod vsock_lib;

#[link(name = "main")]
extern "C" {
    fn server_entry() -> i32;
    fn client_entry() -> i32;
}

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
    let mut result;
    unsafe {
        result = client_entry();
    }
    log::debug!("Client example done: {}\n", result);

    // server::test_server();
    unsafe {
        result = server_entry();
    }

    log::debug!("Server Example done: {}\n", result);
    loop {}
}

use core::panic::PanicInfo;

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
