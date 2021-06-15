// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use rust_firmware_layout::build_time::*;
use rust_firmware_layout::runtime::*;

#[allow(dead_code)]
pub enum SliceType {
    FirmwareIplSlice,
    FirmwarePayloadSlice,
    RuntimePayloadSlice,
    RuntimePayloadHobSlice,
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
            _ => {
                panic!("not support")
            }
        }
    }
}
