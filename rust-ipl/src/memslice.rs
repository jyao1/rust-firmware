// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

const FIRMWARE_IPL_BASE: usize = 0xFFF00000;
const FIRMWARE_IPL_SIZE: usize = 0xF0000;
const FIRMWARE_PAYLOAD_BASE: usize = 0xFFC81000;
const FIRMWARE_PAYLOAD_SIZE: usize = 0x27F000;

const RUNTIME_PAYLOAD_PAGE_BASE: usize = 0x800000;

const RUNTIME_HEAP_SIZE: usize = 0x1000000;
const RUNTIME_STACK_SIZE: usize = 0x800000;
const RUNTIME_PAYLOAD_SIZE: usize = 0x700000;
const RUNTIME_PAYLOAD_HOB_SIZE: usize = 0x1000;

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
                FIRMWARE_IPL_BASE as *const u8,
                FIRMWARE_IPL_SIZE as usize,
            ),
            SliceType::FirmwarePayloadSlice => core::slice::from_raw_parts(
                FIRMWARE_PAYLOAD_BASE as *const u8,
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
                RUNTIME_PAYLOAD_HOB_SIZE as usize,
            ),
            SliceType::RuntimePayloadSlice => {
                core::slice::from_raw_parts_mut(
                    base_address as *const u8 as *mut u8,
                    RUNTIME_PAYLOAD_SIZE
                )
            },
            _ => {
                panic!("not support")
            }
        }
    }
}
