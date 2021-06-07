// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::cmp::min;
use log::*;
use x86_64::{
    // align_down, align_up,
    // registers::control::{Cr3, Cr3Flags},
    // structures::paging::page_table::PageTableEntry,
    structures::paging::PageTableFlags as Flags,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
    },
    PhysAddr,
    VirtAddr,
};

use super::frame::{BMFrameAllocator, FRAME_ALLOCATOR};
use super::consts::PAGE_TABLE_BASE;

pub fn create_mapping(pt: &mut OffsetPageTable, allocator: &mut BMFrameAllocator, mut pa: PhysAddr, mut va: VirtAddr, mut sz: u64) {
    // const ALIGN_4K_BITS: u64 = 12;
    // const ALIGN_4K: u64 = 4096;
    const ALIGN_2M_BITS: u64 = 21;
    const ALIGN_2M: u64 = 1024 * 1024 * 2;
    const ALIGN_1G_BITS: u64 = 30;
    const ALIGN_1G: u64 = 1024 * 1024 * 1024;

    // let allocator: &mut BMFrameAllocator = &mut FRAME_ALLOCATOR.lock();

    while sz > 0 {
        let addr_align = min(pa.as_u64().trailing_zeros(), va.as_u64().trailing_zeros()) as u64;

        let mapped_size = if addr_align >= ALIGN_1G_BITS && sz >= ALIGN_1G {
            trace!(
                "1GB {} {:016x} /{:016x} {:016x}\n",
                addr_align,
                sz,
                pa.as_u64(),
                va.as_u64()
            );
            type S = Size1GiB;
            let page: Page<S> = Page::containing_address(va);
            let frame: PhysFrame<S> = PhysFrame::containing_address(pa);
            let flags = Flags::PRESENT | Flags::WRITABLE;
            unsafe {
                pt.map_to(page, frame, flags, allocator)
                    .expect("map_to failed")
                    .flush();
            }
            S::SIZE
        } else if addr_align >= ALIGN_2M_BITS && sz >= ALIGN_2M {
            trace!(
                "2MB {} {:016x} /{:016x} {:016x}\n",
                addr_align,
                sz,
                pa.as_u64(),
                va.as_u64()
            );
            type S = Size2MiB;
            let page: Page<S> = Page::containing_address(va);
            let frame: PhysFrame<S> = PhysFrame::containing_address(pa);
            let flags = Flags::PRESENT | Flags::WRITABLE;
            unsafe {
                pt.map_to(page, frame, flags, allocator)
                    .expect("map_to failed")
                    .flush();
            }
            S::SIZE
        } else {
            trace!(
                "4KB {} {:016x} /{:016x} {:016x}\n",
                addr_align,
                sz,
                pa.as_u64(),
                va.as_u64()
            );
            type S = Size4KiB;
            let page: Page<S> = Page::containing_address(va);
            let frame: PhysFrame<S> = PhysFrame::containing_address(pa);
            let flags = Flags::PRESENT | Flags::WRITABLE;
            unsafe {
                pt.map_to(page, frame, flags, allocator)
                    .expect("map_to failed")
                    .flush();
            }
            S::SIZE
        };
        sz -= mapped_size;
        pa += mapped_size;
        va += mapped_size;
    }
}

pub fn cr3_write() {
    unsafe {
        x86::controlregs::cr3_write(PAGE_TABLE_BASE);
    }
    log::info!("Cr3 - {:x}\n", unsafe { x86::controlregs::cr3() });
}
