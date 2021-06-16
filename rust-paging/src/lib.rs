// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent
#![no_std]

mod consts;
mod frame;
pub mod paging;

use crate::frame::BMFrameAllocator;
pub use consts::*;

use log::*;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

/// page_table_memory_base: page_table_memory_base
/// system_memory_size
pub fn setup_paging(page_table_memory_base: u64, page_table_size: u64, system_memory_size: u64) {
    // Global variable not writable in rust-firmware environment, using a local allocator
    let mut allocator = BMFrameAllocator::new(page_table_memory_base as usize, page_table_size as usize);

    // The first frame should've already been allocated to level 4 PT
    unsafe { allocator.alloc() };

    info!(
        "Frame allocator init done: {:#x?}\n",
        page_table_memory_base..page_table_memory_base + page_table_size as u64
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
    paging::cr3_write(page_table_memory_base);
}
