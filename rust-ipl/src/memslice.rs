// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use rust_firmware_layout::build_time::*;
use rust_firmware_layout::runtime::*;
use rust_firmware_layout::fsp_build_time::*;

#[allow(dead_code)]
pub enum SliceType {
    FirmwareIplSlice,
    FirmwarePayloadSlice,
    FirmwareFspTSlice,
    FirmwareFspMSlice,
    FirmwareFspSSlice,
    RuntimePayloadSlice,
    RuntimePayloadHobSlice,
    RuntimeStackSlice,
    RuntimeHeapSlice,
}

pub fn get_mem_slice<'a>(t: SliceType) -> &'a [u8] {
    unsafe {
        match t {
            SliceType::FirmwareIplSlice => core::slice::from_raw_parts(
                LOADED_IPL_BASE as *const u8,
                FIRMWARE_IPL_SIZE as usize,
            ),
            SliceType::FirmwarePayloadSlice => core::slice::from_raw_parts(
                LOADED_PAYLOAD_BASE as *const u8,
                FIRMWARE_PAYLOAD_SIZE as usize,
            ),
            SliceType::FirmwareFspTSlice => core::slice::from_raw_parts(
                LOADED_FSP_T_BASE as *const u8,
                FIRMWARE_FSP_T_SIZE as usize,
            ),
            SliceType::FirmwareFspMSlice => core::slice::from_raw_parts(
                LOADED_FSP_M_BASE as *const u8,
                FIRMWARE_FSP_M_SIZE as usize,
            ),
            SliceType::FirmwareFspSSlice => core::slice::from_raw_parts(
                LOADED_FSP_S_BASE as *const u8,
                FIRMWARE_FSP_S_SIZE as usize,
            ),
            _ => {
                panic!("not support")
            }
        }
    }
}

pub fn get_dynamic_mem_slice_mut<'a>(t: SliceType, base_address: usize) -> &'a mut [u8] {
    unsafe {
        match t {
            SliceType::RuntimePayloadHobSlice => core::slice::from_raw_parts_mut(
                base_address as *const u8 as *mut u8,
                RUNTIME_HOB_SIZE as usize,
            ),
            SliceType::RuntimePayloadSlice => core::slice::from_raw_parts_mut(
                base_address as *const u8 as *mut u8,
                RUNTIME_PAYLOAD_SIZE as usize,
            ),
            SliceType::RuntimeStackSlice => core::slice::from_raw_parts_mut(
                base_address as *const u8 as *mut u8,
                RUNTIME_STACK_SIZE as usize,
            ),
            SliceType::RuntimeHeapSlice => core::slice::from_raw_parts_mut(
                base_address as *const u8 as *mut u8,
                RUNTIME_HEAP_SIZE as usize,
            ),
            _ => {
                panic!("not support")
            }
        }
    }
}
