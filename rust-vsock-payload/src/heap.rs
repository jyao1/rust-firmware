// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use uefi_pi::const_guids::MEMORY_ALLOCATION_HEAP_GUID;
use uefi_pi::hob_lib::HobEnums;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the heap allocator.
fn init(heap_start: usize, heap_size: usize) {
    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start, heap_size);
    }
    log::info!(
        "Heap allocator init done: {:#x?}\n",
        heap_start..heap_start + heap_size
    );
}

#[cfg(not(test))]
#[alloc_error_handler]
#[allow(clippy::empty_loop)]
fn alloc_error(_info: core::alloc::Layout) -> ! {
    log::info!("alloc_error ... {:?}\n", _info);
    loop {}
}

fn is_heap_hob(hob: &HobEnums) -> bool {
    match hob {
        HobEnums::MemoryAllocation(memory_allocation) => {
            memory_allocation.alloc_descriptor.name == MEMORY_ALLOCATION_HEAP_GUID
        }
        _ => false,
    }
}

pub fn init_heap(hob_list: &[u8]) -> bool {
    let hob = uefi_pi::hob_lib::HobList::new(hob_list)
        .find(|hob| -> bool { is_heap_hob(hob) })
        .unwrap();
    if let uefi_pi::hob_lib::HobEnums::MemoryAllocation(memory_allocation) = hob {
        let memory_base_address = memory_allocation.alloc_descriptor.memory_base_address;
        let memory_length = memory_allocation.alloc_descriptor.memory_length;
        init(memory_base_address as usize, memory_length as usize);
        true
    } else {
        false
    }
}
