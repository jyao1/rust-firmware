// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

// use rust_firmware_layout::build_time::*;
// use rust_firmware_layout::runtime::*;
use crate::asm;
use crate::fsp_info_header::{FspInfoHeader, FSP_INFO_HEADER_OFF};
use crate::memslice;
// use rust_firmware_layout::fsp_build_time::*;
use scroll::Pread;

const LOADED_FSP_M_BASE: u32 = 0;

///
/// Call FspMemoryInit then return hob
/// TBD: currently copy from rust-ipl. need refactor.
///
pub fn call_fsp_memory_init<'a>() -> Option<&'a [u8]> {
    log::info!("Call FspMemoryInit\n");

    let fsp_m_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspMSlice);
    let fsp_m_info_header = fsp_m_fv_buffer
        .pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF)
        .unwrap();
    let fsp_memory_init =
        (LOADED_FSP_M_BASE + fsp_m_info_header.fsp_memory_init_entry_offset) as usize;

    let fsp_m_upd = &fsp_m_fv_buffer[fsp_m_info_header.cfg_region_offset as usize
        ..(fsp_m_info_header.cfg_region_offset + fsp_m_info_header.cfg_region_size) as usize];

    let mut hob_ptr = core::ptr::null::<u8>();
    let hob_base = &mut hob_ptr;

    log::trace!("Fsp-M-init start\n");
    let res = asm::execute_32bit_code(
        fsp_memory_init as usize,
        fsp_m_upd as *const [u8] as *const u8 as usize,
        hob_base as *mut *const u8 as usize,
    );
    log::trace!("Fsp-M-init done {:#X}, hob_base {:p}\n", res, *hob_base);

    let hob = memslice::get_dynamic_mem_slice_mut(
        memslice::SliceType::RuntimePayloadHobSlice,
        hob_ptr as usize,
    ) as &[u8];

    Some(hob)
}

///
/// Call TempRamExit
/// TBD: currently copy from rust-ipl. need refactor.
///
pub fn call_fsp_m_temp_ram_exit() {
    log::info!("Call TempRamExit\n");

    let fsp_m_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspMSlice);
    let fsp_m_info_header = fsp_m_fv_buffer
        .pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF)
        .unwrap();
    log::trace!("Fsp-M: {:?}\n", fsp_m_info_header);
    let temp_ram_exit_entry =
        (LOADED_FSP_M_BASE + fsp_m_info_header.temp_ram_exit_entry_offset) as usize;

    log::trace!("Fsp-M-temp-ram-exit start\n");
    let res = asm::execute_32bit_code(temp_ram_exit_entry as usize, 0usize, 0usize);
    if res != 0 {
        log::info!("Fsp-M-temp-ram-exit failed {:X}\n", res);
    }
}

///
/// Call FspSiliconInit then return hob
/// TBD: currently copy from rust-ipl. need refactor.
///
pub fn call_fsp_s_silicon_init() {
    log::info!("Call FspSiliconInit\n");
    let fsp_s_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspSSlice);
    let fsp_s_info_header = fsp_s_fv_buffer
        .pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF)
        .unwrap();
    log::trace!("Fsp-S: {:?}\n", fsp_s_info_header);
    let fsp_silicon_init =
        (fsp_s_info_header.image_base + fsp_s_info_header.fsp_silicon_init_entry_offset) as usize;

    let fsp_s_upd = &fsp_s_fv_buffer[fsp_s_info_header.cfg_region_offset as usize
        ..(fsp_s_info_header.cfg_region_offset + fsp_s_info_header.cfg_region_size) as usize];

    log::trace!("Fsp-S-init start\n");
    let res = asm::execute_32bit_code(
        fsp_silicon_init,
        fsp_s_upd as *const [u8] as *const u8 as usize,
        0,
    );
    if res != 0 {
        panic!("FspSiliconInit Failed {:X}", res);
    }
}
