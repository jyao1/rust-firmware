// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

pub mod consts;
pub mod build_time;
pub mod runtime;

pub struct RuntimeMemoryLayout {
    pub runtime_hob_base: u64,
    pub runtime_page_table_base: u64,
    pub runtime_payload_base: u64,
    pub runtime_stack_top: u64,
    pub runtime_stack_base: u64,
    pub runtime_heap_base: u64,
}

impl RuntimeMemoryLayout {
    pub fn new(
        memory_top: u64,
    ) -> Self {
        use crate::runtime::*;
        let current_base = memory_top;

        let current_base = current_base - RUNTIME_HOB_SIZE as u64;
        let runtime_hob_base = current_base;

        let current_base = current_base - RUNTIME_PAGE_TABLE_SIZE as u64;
        let runtime_page_table_base = current_base;

        let current_base = current_base - RUNTIME_PAYLOAD_SIZE as u64;
        let runtime_payload_base = current_base;

        let runtime_stack_top = current_base;
        let current_base = current_base - RUNTIME_STACK_SIZE as u64;
        let runtime_stack_base = current_base;

        let current_base = current_base - RUNTIME_HEAP_SIZE as u64;
        let runtime_heap_base = current_base;

        RuntimeMemoryLayout {
            runtime_hob_base,
            runtime_page_table_base,
            runtime_payload_base,
            runtime_stack_top,
            runtime_stack_base,
            runtime_heap_base,
        }
    }
}

impl fmt::Debug for RuntimeMemoryLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RuntimeMemoryLayout")
            .field("runtime_hob_base", &format_args!("0x{:x}", self.runtime_hob_base))
            .field("runtime_page_table_base", &format_args!("0x{:x}", self.runtime_page_table_base))
            .field("runtime_payload_base", &format_args!("0x{:x}", self.runtime_payload_base))
            .field("runtime_stack_top", &format_args!("0x{:x}", self.runtime_stack_top))
            .field("runtime_stack_base", &format_args!("0x{:x}", self.runtime_stack_base))
            .field("runtime_heap_base", &format_args!("0x{:x}", self.runtime_heap_base))
            .finish()
    }
}
