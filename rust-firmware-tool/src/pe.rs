// Copyright © 2019 Intel Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused)]

use core::mem::size_of;
//use core::mem::transmute;
use crate::mem::MemoryRegion;

const DOS_SIGNATURE: u16 = 0x5a4d;
const PE_SIGNATURE: u32 = 0x00004550;
const MACHINE_I386: u16 = 0x014c;
const MACHINE_X64: u16 = 0x8664;
const OPTIONAL_HDR32_MAGIC: u16 = 0x10b;
const OPTIONAL_HDR64_MAGIC: u16 = 0x20b;
const DIRECTORY_ENTRY_BASERELOC: u32 = 5;

const REL_BASED_ABSOLUTE: u8 = 0;
const REL_BASED_HIGH: u8 = 1;
const REL_BASED_LOW: u8 = 2;
const REL_BASED_HIGHLOW: u8 = 3;
const REL_BASED_HIGHADJ: u8 = 4;
const REL_BASED_DIR64: u8 = 10;

#[repr(packed)]
struct DosHeader {
    e_magic: u16,
    _e_cblp: u16,
    _e_cp: u16,
    _e_crlc: u16,
    _e_cparhdr: u16,
    _e_minalloc: u16,
    _e_maxalloc: u16,
    _e_ss: u16,
    _e_sp: u16,
    _e_csum: u16,
    _e_ip: u16,
    _e_cs: u16,
    _e_lfarlc: u16,
    _e_ovno: u16,
    _e_res: [u16; 4],
    _e_oemid: u16,
    _e_oeminfo: u16,
    _e_res2: [u16; 10],
    e_lfanew: u32,
}

#[repr(packed)]
struct FileHeader {
    machine: u16,
    number_of_sections: u16,
    time_date_stamp: u32,
    pointer_to_symbol_table: u32,
    number_of_symbols: u32,
    size_of_optional_header: u16,
    characteristics: u16,
}

#[repr(packed)]
struct DataDirectory {
    virtual_address: u32,
    size: u32,
}

#[repr(packed)]
struct OptionalHeader32 {
    ///
    /// Standard fields.
    ///
    magic: u16,
    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entrypoint: u32,
    base_of_code: u32,
    base_of_data: u32,
    ///< PE32 contains this additional field, which is absent in PE32+.
    ///
    /// Optional Header Windows-Specific Fields.
    ///
    image_base: u32,
    section_alignment: u32,
    file_alignment: u32,
    major_operating_system_version: u16,
    minor_operating_system_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    win32_version_value: u32,
    size_of_image: u32,
    size_of_headers: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: u32,
    size_of_stack_commit: u32,
    size_of_heap_reserve: u32,
    size_of_heap_commit: u32,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
    data_directory: [DataDirectory; 16],
}

#[repr(packed)]
struct OptionalHeader64 {
    ///
    /// Standard fields.
    ///
    magic: u16,
    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entrypoint: u32,
    base_of_code: u32,
    ///
    /// Optional Header Windows-Specific Fields.
    ///
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_operating_system_version: u16,
    minor_operating_system_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    win32_version_value: u32,
    size_of_image: u32,
    size_of_headers: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
    data_directory: [DataDirectory; 16],
}

#[repr(packed)]
struct PeHeader32 {
    signature: u32,
    file_header: FileHeader,
    optional_header: OptionalHeader32,
}

#[repr(packed)]
struct PeHeader64 {
    signature: u32,
    file_header: FileHeader,
    optional_header: OptionalHeader64,
}

#[repr(packed)]
struct Section {
    name: [u8; 8],
    virtual_size: u32,
    virtual_address: u32,
    size_of_raw_data: u32,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    pointer_to_line_numbers: u32,
    number_of_relocations: u16,
    number_of_line_numbers: u16,
    characteristics: u32,
}

#[repr(packed)]
struct ImageBaseRelocation {
    virtual_address: u32,
    size_of_block: u32,
}

pub fn relocate(
    pe_image: &Vec<u8>,
    new_pe_image: &mut Vec<u8>,
    new_image_base: usize,
) -> Result<(), String> {
    let image_buffer = &pe_image[..];
    let new_image_buffer = &new_pe_image[..];

    let mut loaded_region = MemoryRegion::new(
        new_image_buffer as *const [u8] as *const u8 as usize as u64,
        new_image_buffer.len() as u64,
    );

    println!("relocate to - 0x{:x}", new_image_base);

    let dos_region = MemoryRegion::from_slice(&image_buffer);
    if dos_region.read_u16(0) != DOS_SIGNATURE {
        return Err(String::from("DOS signature error!"));
    }
    let pe_header_offset = dos_region.read_u32(0x3c) as usize;
    let pe_region = MemoryRegion::from_slice(&image_buffer[pe_header_offset..]);
    if pe_region.read_u32(0) != PE_SIGNATURE {
        return Err(String::from("PE signature error!"));
    }
    if pe_region.read_u16(4) != MACHINE_X64 {
        return Err(String::from("PE machine error!"));
    }
    let num_sections = pe_region.read_u16(6) as usize;
    let optional_header_size = pe_region.read_u16(20) as usize;
    let optional_region = MemoryRegion::from_slice(&image_buffer[(24 + pe_header_offset)..]);
    if optional_region.read_u16(0) != OPTIONAL_HDR64_MAGIC {
        return Err(String::from("PE magic error!"));
    }

    let entry_point = optional_region.read_u32(16);
    let image_base = optional_region.read_u64(24);
    let image_size = optional_region.read_u32(56) as usize;
    let size_of_headers = optional_region.read_u32(60) as usize;

    let sections = &image_buffer[(24 + pe_header_offset + optional_header_size)..];
    let sections: &[Section] =
        unsafe { core::slice::from_raw_parts(sections.as_ptr() as *const Section, num_sections) };

    // Load the PE header into the destination memory
    let total_header_size =
        (24 + pe_header_offset + optional_header_size + num_sections * 40) as usize;
    let l: &mut [u8] = loaded_region.as_mut_slice(0, total_header_size as u64);
    l.copy_from_slice(&image_buffer[0..total_header_size]);
    loaded_region.write_u64((24 + pe_header_offset + 24) as u64, new_image_base as u64);

    for section in sections {
        for x in 0..section.virtual_size {
            loaded_region.write_u8((x + section.virtual_address) as u64, 0);
        }
        let section_size = core::cmp::min(section.size_of_raw_data, section.virtual_size);
        let l: &mut [u8] =
            loaded_region.as_mut_slice(section.virtual_address as u64, section_size as u64);
        l.copy_from_slice(
            &image_buffer[section.pointer_to_raw_data as usize
                ..(section.pointer_to_raw_data + section_size) as usize],
        );
    }

    // Relocate image in the destination memory
    for section in sections {
        if &section.name[0..6] == b".reloc" {
            let section_size = core::cmp::min(section.size_of_raw_data, section.virtual_size);
            let l: &mut [u8] =
                loaded_region.as_mut_slice(section.virtual_address as u64, section_size as u64);

            let reloc_region = MemoryRegion::from_slice(l);

            let mut section_bytes_remaining = section_size;
            let mut offset = 0;
            while section_bytes_remaining > 0 {
                // Read details for block
                let page_rva = reloc_region.read_u32(offset);
                let block_size = reloc_region.read_u32(offset + 4);
                let mut block_offset = 8;
                while block_offset < block_size {
                    let entry = reloc_region.read_u16(offset + u64::from(block_offset));

                    let entry_type = (entry >> 12) as u8;
                    let entry_offset = (entry & 0xfff) as u32;

                    match entry_type {
                        REL_BASED_DIR64 => {
                            let location = (page_rva + entry_offset) as u64;
                            let value = loaded_region.read_u64(location);
                            loaded_region
                                .write_u64(location, value - image_base + new_image_base as u64);
                        }
                        REL_BASED_ABSOLUTE => {}
                        _ => {
                            panic!("unknown reloc type");
                        }
                    }
                    block_offset += 2;
                }

                section_bytes_remaining -= block_size;
                offset += u64::from(block_size);
            }
        }
    }

    println!("relocation success!");

    Ok(())
}
