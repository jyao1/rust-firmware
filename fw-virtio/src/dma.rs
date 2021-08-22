// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::{Result, VirtioError};

type VirtAddr = usize;
type PhysAddr = usize;

pub(crate) const PAGE_SIZE: usize = 4096;

pub struct DMA {
    paddr: u32,
    pages: u32,
}

impl DMA {
    pub fn new(pages: usize) -> Result<Self> {
        let paddr = unsafe { virtio_dma_alloc(pages) };
        if paddr == 0 {
            return Err(VirtioError::DmaError);
        }
        Ok(DMA {
            paddr: paddr as u32,
            pages: pages as u32,
        })
    }

    pub fn paddr(&self) -> usize {
        self.paddr as usize
    }

    pub fn vaddr(&self) -> usize {
        phys_to_virt(self.paddr as usize)
    }
}

impl Drop for DMA {
    fn drop(&mut self) {
        let err = unsafe { virtio_dma_dealloc(self.paddr as usize, self.pages as usize) };
        assert_eq!(err, 0, "failed to deallocate DMA");
    }
}

pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    unsafe { virtio_phys_to_virt(paddr) }
}

pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    unsafe { virtio_virt_to_phys(vaddr) }
}

extern "C" {
    fn virtio_dma_alloc(pages: usize) -> PhysAddr;
    fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32;
    fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr;
    fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr;
}
