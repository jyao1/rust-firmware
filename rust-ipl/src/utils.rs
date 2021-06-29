// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use r_uefi_pi::fv;
use rust_firmware_layout::consts::*;
use uefi_pi::hob_lib;

pub fn efi_size_to_page(size: u64) -> u64 {
    (size + SIZE_4K - 1) / SIZE_4K
}

pub fn efi_page_to_size(page: u64) -> u64 {
    page * SIZE_4K
}

/// flag  ture align to low address else high address
pub fn align_value(value: u64, align: u64, flag: bool) -> u64 {
    if flag {
        value & ((!(align - 1)) as u64)
    } else {
        value - (value & (align - 1)) as u64 + align
    }
}

pub fn find_and_report_entry_point(
    firmware_buffer: &[u8],
    loaded_buffer: &mut [u8],
) -> (u64, u64, u64) {
    let image = uefi_pi::fv_lib::get_image_from_fv(
        firmware_buffer,
        fv::FV_FILETYPE_DXE_CORE,
        fv::SECTION_PE32,
    )
    .unwrap();
    log::trace!("found image len is: {:x}\n", image.len());
    log::trace!(
        "loaded_buffer addr: {:x}\n",
        loaded_buffer as *const [u8] as *const u8 as usize
    );
    if elf_loader::elf::is_elf(image) {
        log::info!("Payload is elf image\n");
        elf_loader::elf::relocate_elf(image, loaded_buffer)
    } else if pe_loader::pe::is_pe(image) {
        log::info!("Payload is pe image\n");
        pe_loader::pe::relocate_pe_mem(image, loaded_buffer)
    } else {
        panic!("format not support")
    }
}

pub fn dump_hob_buffer(hob_list: &[u8]) {
    let hob_size = hob_lib::get_hob_total_size(hob_list).unwrap();
    log::trace!("hob_size: {}\n", hob_size);
    for (index, value) in hob_list[..hob_size].iter().enumerate() {
        if index % 16 == 0 {
            log::trace!("\n");
        }
        log::trace!("0x{:02X}, ", value);
    }
    log::trace!("\n");
}
