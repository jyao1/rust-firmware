// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(global_asm)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

mod asm;
mod memslice;
mod const_guids;
mod utils;

use r_efi::efi;
use r_uefi_pi::hob;
use rust_firmware_layout::consts::SIZE_4K;
use uefi_pi::hob_lib;

use rust_firmware_layout::build_time::*;
use rust_firmware_layout::runtime::*;

use rust_firmware_layout::RuntimeMemoryLayout;

use rust_fsp_wrapper::fsp_info_header::{FSP_INFO_HEADER_OFF, FspInfoHeader};
use rust_fsp_wrapper::fsp_m_udp::FspmUpd;
use rust_fsp_wrapper::fsp_s_udp::FspsUpd;

use scroll::{Pread, Pwrite};

#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct HobTemplate {
    pub handoff_info_table: hob::HandoffInfoTable,
    pub firmware_volume: hob::FirmwareVolume,
    pub cpu: hob::Cpu,
    pub hypervisor_fw: hob::MemoryAllocation,
    pub page_table: hob::MemoryAllocation,
    pub stack: hob::MemoryAllocation,
    pub memory_above1_m: hob::ResourceDescription,
    pub memory_blow1_m: hob::ResourceDescription,
    pub end_off_hob: hob::GenericHeader,
}

#[cfg(target_os="uefi")]
use core::panic::PanicInfo;

#[cfg(target_os="uefi")]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(info: &PanicInfo) -> ! {
    log::info!("panic ... {:?}\n", info);
    loop {}
}

#[cfg(target_os="uefi")]
#[alloc_error_handler]
#[allow(clippy::empty_loop)]
fn alloc_error(_info: core::alloc::Layout) -> ! {
    log::info!("alloc_error ... {:?}\n", _info);
    loop {}
}

#[no_mangle]
#[export_name = "efi_main"]
pub extern "win64" fn _start(
    temp_ram_base: usize,
    temp_ram_top: usize,
    stack_top_or_temp_page_table_base: usize,
    initial_eax_value: usize,
) -> ! {
    let boot_fv = LOADED_IPL_BASE;
    log::info!(
        "Starting RUST Based IPL:
    Boot_fv - {:#X}
    Top of stack - {:#X}
    Temp ram base - {:#X}
    Temp ram top - {:#X}
    Temp page table base - {:#X}
    Initial eax value - {:#X}\n",
        boot_fv,
        stack_top_or_temp_page_table_base,
        temp_ram_base,
        temp_ram_top,
        stack_top_or_temp_page_table_base,
        initial_eax_value,
    );

    // fw_exception::setup_exception_handlers();
    // log::info!("setup_exception_handlers done\n");

    dump_fsp_t_info();

    let hob_list = call_fsp_memory_init().expect("memory init failed");

    // top of low usable memory
    let memory_tolum = hob_lib::get_system_memory_size_below_4gb(hob_list);
    log::trace!("memory lotum 0: {:#X}\n", memory_tolum);

    let runtime_memory_layout = RuntimeMemoryLayout::new(memory_tolum);

    // switch_stack
    log::info!("Switch to stack: {:#X}\n", runtime_memory_layout.runtime_stack_top);
    asm::switch_stack(continue_function as usize, runtime_memory_layout.runtime_stack_top as usize, hob_list as *const [u8] as *const u8 as usize, stack_top_or_temp_page_table_base);

    unreachable!();
}

pub extern "win64" fn continue_function(hob_address: usize, _tmp_stack_top: usize) -> ! {
    log::info!("Continue function - hob_address: {:#X}\n", hob_address);

    let fsp_hob_list = memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadHobSlice, hob_address);
    let memory_tolum = hob_lib::get_system_memory_size_below_4gb(fsp_hob_list);
    log::trace!("memory lotum 1: {:#X}\n", memory_tolum);
    let runtime_memory_layout = RuntimeMemoryLayout::new(memory_tolum);

    // Set host Paging
    let memory_size = 0x1000000000; // TODO: hardcoding to 64GiB for now
    paging::setup_paging(
        runtime_memory_layout.runtime_page_table_base as u64,
        RUNTIME_PAGE_TABLE_SIZE as u64,
        memory_size,
    );
    log::info!("Migrate pagetable at {:#X}\n", runtime_memory_layout.runtime_page_table_base);

    call_fsp_m_temp_ram_exit();

    call_fsp_s_silicon_init();
    let memory_tolum = hob_lib::get_system_memory_size_below_4gb(fsp_hob_list);
    log::trace!("memory lotum 2: {:#X}\n", memory_tolum);

    transfer_to_payload(&runtime_memory_layout, fsp_hob_list);

    unreachable!();
}

fn dump_fsp_t_info() {
    let fsp_t_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspTSlice);
    let fsp_t_info_header = fsp_t_fv_buffer.pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF).unwrap();
    log::trace!("Fsp-T: {:?}\n", fsp_t_info_header);
}

///
/// Call FspMemoryInit then return hob
///
fn call_fsp_memory_init<'a>() -> Option<&'a [u8]> {
    log::info!("Call FspMemoryInit\n");

    let mut hob_ptr = core::ptr::null::<u8>();
    let hob_base = &mut hob_ptr;

    let mut fsp_m_upd_new = [0u8; core::mem::size_of::<FspmUpd>()];

    let fsp_m_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspMSlice);
    let fsp_m_info_header = fsp_m_fv_buffer.pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF).unwrap();

    let fsp_memory_init = (LOADED_FSP_M_BASE + fsp_m_info_header.fsp_memory_init_entry_offset) as usize;

    let mut fsp_m_upd = fsp_m_fv_buffer[fsp_m_info_header.cfg_region_offset as usize..(fsp_m_info_header.cfg_region_offset + fsp_m_info_header.cfg_region_size) as usize]
    .pread::<FspmUpd>(0)
    .unwrap();
    log::trace!("Fsp-M-upd: {:?}\n", fsp_m_upd);
    fsp_m_upd.fspm_arch_upd.nvs_buffer_ptr = 0;
    fsp_m_upd_new.pwrite(fsp_m_upd, 0).unwrap();

    log::trace!("Fsp-M-init start\n");
    let res = asm::execute_32bit_code(fsp_memory_init as usize, &fsp_m_upd_new[..] as *const [u8] as *const u8 as usize, hob_base as *mut *const u8 as usize);
    log::trace!("Fsp-M-init done {:#X}, hob_base {:p}\n", res, *hob_base);

    let hob = memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadHobSlice, hob_ptr as usize) as &[u8];

    Some(hob)
}

///
/// Call TempRamExit
///
fn call_fsp_m_temp_ram_exit() {
    log::info!("Call TempRamExit\n");

    let fsp_m_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspMSlice);
    let fsp_m_info_header = fsp_m_fv_buffer.pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF).unwrap();
    log::trace!("Fsp-M: {:?}\n", fsp_m_info_header);
    let temp_ram_exit_entry = (LOADED_FSP_M_BASE + fsp_m_info_header.temp_ram_exit_entry_offset) as usize;

    log::trace!("Fsp-M-temp-ram-exit start\n");
    let res = asm::execute_32bit_code(temp_ram_exit_entry as usize, 0usize, 0usize);
    if res != 0 {
        log::info!("Fsp-M-temp-ram-exit failed {:X}\n", res);
    }
}

///
/// Call FspSiliconInit then return hob
///
fn call_fsp_s_silicon_init() {
    log::info!("Call FspSiliconInit\n");
    let fsp_s_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwareFspSSlice);
    let fsp_s_info_header =  fsp_s_fv_buffer.pread::<FspInfoHeader>(FSP_INFO_HEADER_OFF).unwrap();
    log::trace!("Fsp-S: {:?}\n", fsp_s_info_header);
    let fsp_silicon_init = (fsp_s_info_header.image_base + fsp_s_info_header.fsp_silicon_init_entry_offset) as usize;

    let fsp_s_upd = fsp_s_fv_buffer[fsp_s_info_header.cfg_region_offset as usize..(fsp_s_info_header.cfg_region_offset + fsp_s_info_header.cfg_region_size) as usize]
    .pread::<FspsUpd>(0)
    .expect("fsp_s_upd get failed");

    let mut fsp_s_upd_new =  [0u8; core::mem::size_of::<FspsUpd>()];
    fsp_s_upd_new.pwrite(fsp_s_upd, 0).expect("fsp_s_upd_write_failed");


    log::trace!("Fsp-S-init start\n");
    let res = asm::execute_32bit_code(fsp_silicon_init, &fsp_s_upd_new[..] as *const [u8] as *const u8 as usize, 0);
    if res != 0 {
        panic!("FspSiliconInit Failed {:X}", res);
    }
}

fn transfer_to_payload(runtime_memory_layout: &RuntimeMemoryLayout, fsp_hob_list: &mut [u8]) {
    hob_lib::dump_hob(fsp_hob_list);

    let loaded_buffer = memslice::get_dynamic_mem_slice_mut(
        memslice::SliceType::RuntimePayloadSlice,
        runtime_memory_layout.runtime_payload_base as usize,
    );
    let payload_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwarePayloadSlice);
    log::trace!(
        "payload_fv_start: {:#X}\n",
        payload_fv_buffer as *const [u8] as *const u8 as usize
    );

    let (payload_entry
        , basefw, basefwsize) =
        utils::find_and_report_entry_point(payload_fv_buffer, loaded_buffer);
    log::trace!("payload basefw, size: {:#X}, {:#X}", utils::align_value(basefw, SIZE_4K, true), basefwsize);
    let payload_entry = payload_entry as usize;

    migrate_hobs(runtime_memory_layout, fsp_hob_list);
    log::info!("Migrate hobs at: {:#X}\n", runtime_memory_layout.runtime_hob_base);

    log::info!("Call payload entry: {:#X}\n", payload_entry);
    asm::switch_stack(
        payload_entry,
        runtime_memory_layout.runtime_stack_top as usize,
        runtime_memory_layout.runtime_hob_base as usize,
        0,
    );
    unreachable!()
}

fn migrate_hobs(runtime_memory_layout: &RuntimeMemoryLayout, fsp_hobs: &[u8]) {

    let migrated_hob_list = memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadHobSlice, runtime_memory_layout.runtime_hob_base as usize);
    migrated_hob_list[..fsp_hobs.len()].copy_from_slice(fsp_hobs);

    let page_table_hob = hob::MemoryAllocation {
        header: hob::GenericHeader::new(
            hob::HobType::MEMORY_ALLOCATION,
            core::mem::size_of::<hob::MemoryAllocation>()
        ),
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: const_guids::PAGE_TABLE_NAME_GUID,
            memory_base_address: runtime_memory_layout.runtime_page_table_base,
            memory_length: RUNTIME_PAGE_TABLE_SIZE as u64,
            memory_type: efi::MemoryType::BootServicesData as u32,
            reserved: [0u8; 4],
        },
    };
    add_memory_allocation_to_ipl_hobs(&runtime_memory_layout, page_table_hob);

    let hypervisor_fw_hob = hob::MemoryAllocation {
        header: hob::GenericHeader::new(hob::HobType::MEMORY_ALLOCATION, core::mem::size_of::<hob::MemoryAllocation>()) ,
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: const_guids::HYPERVISORFW_NAME_GUID,
            memory_base_address: runtime_memory_layout.runtime_payload_base,
            memory_length: utils::efi_page_to_size(utils::efi_size_to_page(FIRMWARE_PAYLOAD_SIZE as u64)),
            memory_type: efi::MemoryType::BootServicesCode as u32,
            reserved: [0u8; 4],
        },
    };
    add_memory_allocation_to_ipl_hobs(&runtime_memory_layout, hypervisor_fw_hob);

    let stack_hob = hob::MemoryAllocation {
        header: hob::GenericHeader::new(hob::HobType::MEMORY_ALLOCATION, core::mem::size_of::<hob::MemoryAllocation>()) ,
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: const_guids::MEMORY_ALLOCATION_STACK_GUID,
            memory_base_address: runtime_memory_layout.runtime_stack_base as u64,
            memory_length: RUNTIME_STACK_SIZE as u64,
            memory_type: efi::MemoryType::BootServicesData as u32,
            reserved: [0u8; 4],
        },
    };
    add_memory_allocation_to_ipl_hobs(&runtime_memory_layout, stack_hob);

    let firmware_volume = hob::FirmwareVolume {
        header: hob::GenericHeader::new(hob::HobType::FV, core::mem::size_of::<hob::FirmwareVolume>()),
        base_address: LOADED_RESERVED1_BASE as u64,
        length: FIRMWARE_SIZE as u64,
    };
    add_firmware_to_ipl_hobs(&runtime_memory_layout, firmware_volume);

    utils::dump_hob_buffer(migrated_hob_list);
}

fn add_hob_to_ipl_hobs(runtime_memory_layout: &RuntimeMemoryLayout, hob_buffer: &[u8]) -> bool {
    let hob_list = memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadHobSlice, runtime_memory_layout.runtime_hob_base as usize);
    let mut hl = hob_lib::HobListMut::new(hob_list);
    hl.add(hob_buffer)
}

fn add_memory_allocation_to_ipl_hobs(runtime_memory_layout: &RuntimeMemoryLayout, hob: hob::MemoryAllocation) {
    let write_hob_buffer = &mut [0u8; core::mem::size_of::<hob::MemoryAllocation>()][..];
    write_hob_buffer.pwrite::<hob::MemoryAllocation>(hob, 0).expect("write memory allocation hob failed");
    add_hob_to_ipl_hobs(runtime_memory_layout, write_hob_buffer);
}

fn add_firmware_to_ipl_hobs(runtime_memory_layout: &RuntimeMemoryLayout, hob: hob::FirmwareVolume) {
    let write_hob_buffer = &mut [0u8; core::mem::size_of::<hob::FirmwareVolume>()][..];
    write_hob_buffer.pwrite::<hob::FirmwareVolume>(hob, 0).expect("write fv hob failed");
    add_hob_to_ipl_hobs(runtime_memory_layout, write_hob_buffer);
}
