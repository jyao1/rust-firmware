// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![allow(unused)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(global_asm)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

mod memslice;
mod pci;
mod sec;
mod asm;

use r_efi::efi;
use r_uefi_pi::pi;
use r_uefi_pi::hob;

use core::ffi::c_void;
use core::panic::PanicInfo;

#[macro_use]
use fw_logger::*;

use rust_firmware_layout::runtime::*;
use rust_firmware_layout::RuntimeMemoryLayout;
use rust_firmware_layout::build_time::*;

use scroll::{Pread, Pwrite};

#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct HobTemplate {
    pub handoff_info_table: hob::HandoffInfoTable,
    pub firmware_volume: hob::FirmwareVolume,
    pub cpu: hob::Cpu,
    pub hypervisor_fw: hob::MemoryAllocation,
    pub page_table: hob::MemoryAllocation,
    pub stack: hob::MemoryAllocation,
    pub memory_above1_m: hob::ResourceDescription,
    pub memory_blow1_m: hob::ResourceDescription,
    pub end_off_hob: hob::Header,
}

// #[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &PanicInfo) -> ! {
    log!("panic ... {:?}\n", _info);
    //log!("panic...");
    loop {}
}

// #[cfg(not(test))]
#[alloc_error_handler]
#[allow(clippy::empty_loop)]
fn alloc_error(_info: core::alloc::Layout) -> ! {
    log!("alloc_error ... {:?}\n", _info);
    loop {}
}

#[no_mangle]
#[export_name = "efi_main"]
pub extern "win64" fn _start(boot_fv: *const c_void, top_of_stack: *const c_void) -> ! {
    log!(
        "Starting RUST Based IPL boot_fv - {:p}, Top of stack - {:p} \n",
        boot_fv,
        top_of_stack
    );

    fw_exception::setup_exception_handlers();
    log!("setup_exception_handlers done\n");

    let memory_top = sec::get_system_memory_size_below4_gb();

    let runtime_memory_layout = RuntimeMemoryLayout::new(memory_top);
    log!("runtime memory layout: {:?}\n", runtime_memory_layout);

    log!(
        " PcdOvmfDxeMemFvBase: 0x{:X}\n",
        LOADED_PAYLOAD_BASE,
    );
    log!(
        " PcdOvmfDxeMemFvSize: 0x{:X}\n",
        FIRMWARE_PAYLOAD_SIZE,
    );
    log!(
        " PcdOvmfPeiMemFvBase: 0x{:X}\n",
        LOADED_IPL_BASE,
    );
    log!(
        " PcdOvmfPeiMemFvSize: 0x{:X}\n",
        FIRMWARE_IPL_SIZE,
    );

    sec::set_apic_mode(sec::LOCAL_APIC_MODE_X2APIC);
    sec::initialize_apic_timer(
        sec::TimerDivide::Div2,
        0xffffffffu32,
        sec::TimerMode::Periodic,
        5u8,
    );
    sec::disable_apic_timer_interrupt();
    log!(" SetApicMode: Done\n");

    let mut hob_header = hob::Header {
        r#type: hob::HOB_TYPE_END_OF_HOB_LIST,
        length: core::mem::size_of::<hob::Header>() as u16,
        reserved: 0,
    };

    let memory_bottom = memory_top - sec::SIZE_16MB;

    let mut handoff_info_table = hob::HandoffInfoTable {
        header: hob::Header {
            r#type: hob::HOB_TYPE_HANDOFF,
            length: core::mem::size_of::<hob::HandoffInfoTable>() as u16,
            reserved: 0,
        },
        version: 9u32,
        boot_mode: pi::boot_mode::BOOT_WITH_FULL_CONFIGURATION,
        efi_memory_top: memory_top,
        efi_memory_bottom: memory_bottom,
        efi_free_memory_top: memory_top,
        efi_free_memory_bottom: memory_bottom
            + sec::efi_page_to_size(sec::efi_size_to_page(
                core::mem::size_of::<HobTemplate>() as u64
            )),
        efi_end_of_hob_list: runtime_memory_layout.runtime_hob_base + core::mem::size_of::<HobTemplate>() as u64,
    };


    let mut cpu = hob::Cpu {
        header: hob::Header {
            r#type: hob::HOB_TYPE_CPU,
            length: core::mem::size_of::<hob::Cpu>() as u16,
            reserved: 0,
        },
        size_of_memory_space: sec::cpu_get_memory_space_size(), // TBD asmcpuid
        size_of_io_space: 16u8,
        reserved: [0u8; 6],
    };


    let mut firmware_volume = hob::FirmwareVolume {
        header: hob::Header {
            r#type: hob::HOB_TYPE_FV,
            length: core::mem::size_of::<hob::FirmwareVolume>() as u16,
            reserved: 0,
        },
        base_address: boot_fv as u64 - FIRMWARE_PAYLOAD_SIZE as u64,
        length: FIRMWARE_PAYLOAD_SIZE as u64,
    };

    const MEMORY_ALLOCATION_STACK_GUID: efi::Guid = efi::Guid::from_fields(
        0x4ED4BF27,
        0x4092,
        0x42E9,
        0x80,
        0x7D,
        &[0x52, 0x7B, 0x1D, 0x00, 0xC9, 0xBD],
    );
    let mut stack = hob::MemoryAllocation {
        header: hob::Header {
            r#type: hob::HOB_TYPE_MEMORY_ALLOCATION,
            length: core::mem::size_of::<hob::MemoryAllocation>() as u16,
            reserved: 0,
        },
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: *MEMORY_ALLOCATION_STACK_GUID.as_bytes(),
            memory_base_address: runtime_memory_layout.runtime_stack_base as u64,
            memory_length: RUNTIME_STACK_SIZE as u64,
            memory_type: efi::MemoryType::BootServicesData as u32,
            reserved: [0u8; 4],
        },
    };

    // Enable host Paging
    const PAGE_TABLE_NAME_GUID: efi::Guid = efi::Guid::from_fields(
        0xF8E21975,
        0x0899,
        0x4F58,
        0xA4,
        0xBE,
        &[0x55, 0x25, 0xA9, 0xC6, 0xD7, 0x7A],
    );
    let memory_size = 0x1000000000; // TODO: hardcoding to 64GiB for now

    paging::setup_paging(
        runtime_memory_layout.runtime_page_table_base as u64,
        RUNTIME_PAGE_TABLE_SIZE as u64,
        memory_size,
    );

    let mut page_table = hob::MemoryAllocation {
        header: hob::Header {
            r#type: hob::HOB_TYPE_MEMORY_ALLOCATION,
            length: core::mem::size_of::<hob::MemoryAllocation>() as u16,
            reserved: 0,
        },
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: *PAGE_TABLE_NAME_GUID.as_bytes(),
            memory_base_address: runtime_memory_layout.runtime_page_table_base,
            memory_length: RUNTIME_PAGE_TABLE_SIZE as u64,
            memory_type: efi::MemoryType::BootServicesData as u32,
            reserved: [0u8; 4],
        },
    };

    const DEFAULT_GUID: efi::Guid = efi::Guid::from_fields(
        0x4ED4BF27,
        0x4092,
        0x42E9,
        0x80,
        0x7D,
        &[0x52, 0x7B, 0x1D, 0x00, 0xC9, 0xBD],
    );
    let lowmemory = sec::get_system_memory_size_below4_gb();

    let mut memory_above1_m = hob::ResourceDescription {
        header: hob::Header {
            r#type: hob::HOB_TYPE_RESOURCE_DESCRIPTOR,
            length: core::mem::size_of::<hob::ResourceDescription>() as u16,
            reserved: 0,
        },
        owner: *efi::Guid::from_fields(
            0x4ED4BF27,
            0x4092,
            0x42E9,
            0x80,
            0x7D,
            &[0x52, 0x7B, 0x1D, 0x00, 0xC9, 0xBD],
        ).as_bytes(),
        resource_type: hob::RESOURCE_SYSTEM_MEMORY,
        resource_attribute: hob::RESOURCE_ATTRIBUTE_PRESENT
            | hob::RESOURCE_ATTRIBUTE_INITIALIZED
            | hob::RESOURCE_ATTRIBUTE_UNCACHEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_COMBINEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_THROUGH_CACHEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_BACK_CACHEABLE
            | hob::RESOURCE_ATTRIBUTE_TESTED,
        physical_start: 0x100000u64,
        resource_length: lowmemory - 0x100000u64,
    };


    let mut memory_below1_m = hob::ResourceDescription {
        header: hob::Header {
            r#type: hob::HOB_TYPE_RESOURCE_DESCRIPTOR,
            length: core::mem::size_of::<hob::ResourceDescription>() as u16,
            reserved: 0,
        },
        owner: *efi::Guid::from_fields(
            0x4ED4BF27,
            0x4092,
            0x42E9,
            0x80,
            0x7D,
            &[0x52, 0x7B, 0x1D, 0x00, 0xC9, 0xBD],
        ).as_bytes(),
        resource_type: hob::RESOURCE_SYSTEM_MEMORY,
        resource_attribute: hob::RESOURCE_ATTRIBUTE_PRESENT
            | hob::RESOURCE_ATTRIBUTE_INITIALIZED
            | hob::RESOURCE_ATTRIBUTE_UNCACHEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_COMBINEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_THROUGH_CACHEABLE
            | hob::RESOURCE_ATTRIBUTE_WRITE_BACK_CACHEABLE
            | hob::RESOURCE_ATTRIBUTE_TESTED,
        physical_start: 0u64,
        resource_length: 0x80000u64 + 0x20000u64,
    };

    let loaded_buffer =
        memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadSlice, runtime_memory_layout.runtime_payload_base as usize);

    let payload_fv_buffer = memslice::get_mem_slice(memslice::SliceType::FirmwarePayloadSlice);
    log!("payload_fv_start: 0x{:X}\n", payload_fv_buffer as *const [u8] as *const u8 as usize);
    let (entry, basefw, basefwsize) = sec::find_and_report_entry_point(
        payload_fv_buffer,
        loaded_buffer,
    );
    let entry = entry as usize;

    const HYPERVISORFW_NAME_GUID: efi::Guid = efi::Guid::from_fields(
        0x6948d4a,
        0xd359,
        0x4721,
        0xad,
        0xf6,
        &[0x52, 0x25, 0x48, 0x5a, 0x6a, 0x3a],
    );


    let mut hypervisor_fw = hob::MemoryAllocation {
        header: hob::Header {
            r#type: hob::HOB_TYPE_MEMORY_ALLOCATION,
            length: core::mem::size_of::<hob::MemoryAllocation>() as u16,
            reserved: 0,
        },
        alloc_descriptor: hob::MemoryAllocationHeader {
            name: *HYPERVISORFW_NAME_GUID.as_bytes(),
            memory_base_address: basefw,
            memory_length: sec::efi_page_to_size(sec::efi_size_to_page(basefwsize)),
            memory_type: efi::MemoryType::BootServicesCode as u32,
            reserved: [0u8; 4],
        },
    };

    let mut hob_template = HobTemplate {
        handoff_info_table,
        firmware_volume,
        cpu,
        hypervisor_fw,
        page_table,
        stack,
        memory_above1_m,
        memory_blow1_m: memory_below1_m,
        end_off_hob: hob::Header {
            r#type: hob::HOB_TYPE_END_OF_HOB_LIST,
            length: core::mem::size_of::<hob::Header>() as u16,
            reserved: 0,
        },
    };
    log!(" Hob prepare\n");

    //
    // Clear 8259 interrupt
    //
    unsafe {
        x86::io::outb(0x21u16, 0xffu8);
        x86::io::outb(0xA1u16, 0xffu8);
    }

    //
    // Disable A20 Mask
    //
    unsafe {
        let res = x86::io::inb(0x92u16);
        x86::io::outb(0x92u16, res | 0b10 as u8);
    }

    pci::initialize_acpi_pm();
    sec::pci_ex_bar_initialization();
    sec::init_pci();
    sec::virt_io_blk();

    let hob_base = runtime_memory_layout.runtime_hob_base as usize;
    let hob = memslice::get_dynamic_mem_slice_mut(memslice::SliceType::RuntimePayloadHobSlice, hob_base);

    let _res = hob.pwrite(hob_template, 0).expect("write hob failed!");

    log!("payload entry is: 0x{:X}\n", entry);
    asm::switch_stack(entry, runtime_memory_layout.runtime_stack_top as usize, hob_base, 0);
    loop {}
}
