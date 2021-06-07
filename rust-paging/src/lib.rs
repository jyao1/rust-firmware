// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
#![no_std]

mod consts;
mod frame;
pub mod paging;

pub use consts::*;
use crate::frame::BMFrameAllocator;

use log::*;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

pub fn init() {
    frame::init();
}

/// page_table_memory_base: page_table_memory_base
/// system_memory_size
pub fn setup_paging(page_table_memory_base: u64, system_memory_size: u64) {
    // Global variable not writable in rust-firmware environment, using a local allocator
    let mut allocator = BMFrameAllocator::new(PAGE_TABLE_BASE as usize, PAGE_TABLE_SIZE);

    // The first frame should've already been allocated to level 4 PT
    unsafe { allocator.alloc() };

    info!(
        "Frame allocator init done: {:#x?}\n",
        PAGE_TABLE_BASE..PAGE_TABLE_BASE + PAGE_TABLE_SIZE as u64
    );

    let mut pt = unsafe {
        OffsetPageTable::new(
            &mut *(page_table_memory_base as *mut PageTable),
            VirtAddr::new(PHYS_VIRT_OFFSET as u64),
        )
    };
    paging::create_mapping(
        &mut pt,
        &mut allocator,
        PhysAddr::new(0),
        VirtAddr::new(0),
        system_memory_size,
    );
    paging::cr3_write();
}
