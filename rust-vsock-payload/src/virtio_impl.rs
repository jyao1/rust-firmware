// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use log::trace;

static mut DMA_PADDR: usize = 0x0;
static mut DMA_END: usize = 0x0;

pub fn init(dma_base: usize, dma_size: usize) {
    unsafe {
        DMA_PADDR = dma_base;
        DMA_END = dma_base + dma_size;
    }
}

#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let paddr = unsafe {
        if DMA_PADDR == 0
            || DMA_PADDR
                .checked_add(0x1000 * pages)
                .expect("dma alloc failed")
                >= DMA_END
        {
            panic!("DMA need be init");
        }
        DMA_PADDR
    };
    unsafe { DMA_PADDR += 0x1000 * pages }
    trace!("alloc DMA: paddr={:#x}, pages={}\n", paddr, pages);
    paddr
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
    trace!("dealloc DMA: paddr={:#x}, pages={}\n", paddr, pages);
    0
}

#[no_mangle]
extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr
}

type VirtAddr = usize;
type PhysAddr = usize;
