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

use core::ffi::c_void;
use core::mem::transmute;
use core::mem::size_of;

use crate::mem::MemoryRegion;

pub const IMAGE_DOS_SIGNATURE:     u16 = 0x5A4D; // 'M', 'Z'

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageDosHeader {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: u32,
}

pub const IMAGE_FILE_MACHINE_I386:           u16 = 0x014c;
pub const IMAGE_FILE_MACHINE_IA64:           u16 = 0x0200;
pub const IMAGE_FILE_MACHINE_EBC:            u16 = 0x0EBC;
pub const IMAGE_FILE_MACHINE_X64:            u16 = 0x8664;
pub const IMAGE_FILE_MACHINE_ARMTHUMB_MIXED: u16 = 0x01c2;
pub const IMAGE_FILE_MACHINE_ARM64:          u16 = 0xAA64;

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageFileHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

pub const IMAGE_NUMBER_OF_DIRECTORY_ENTRIES: usize = 16;

pub const IMAGE_DIRECTORY_ENTRY_EXPORT:      usize = 0;
pub const IMAGE_DIRECTORY_ENTRY_IMPORT:      usize = 1;
pub const IMAGE_DIRECTORY_ENTRY_RESOURCE:    usize = 2;
pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION:   usize = 3;
pub const IMAGE_DIRECTORY_ENTRY_SECURITY:    usize = 4;
pub const IMAGE_DIRECTORY_ENTRY_BASERELOC:   usize = 5;
pub const IMAGE_DIRECTORY_ENTRY_DEBUG:       usize = 6;
pub const IMAGE_DIRECTORY_ENTRY_COPYRIGHT:   usize = 7;
pub const IMAGE_DIRECTORY_ENTRY_GLOBALPTR:   usize = 8;
pub const IMAGE_DIRECTORY_ENTRY_TLS:         usize = 9;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: usize = 10;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct ImageDataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

pub const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x10b;
pub const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x20b;

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageOptionalHeader32 {
    // standard field
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    // optional
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [ImageDataDirectory ; IMAGE_NUMBER_OF_DIRECTORY_ENTRIES]
}

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageOptionalHeader64 {
    // standard field
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    // optional
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [ImageDataDirectory ; IMAGE_NUMBER_OF_DIRECTORY_ENTRIES]
}

pub const IMAGE_PE_SIGNATURE:     u32 = 0x004550; // 'P', 'E', '\0', '\0'

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageNtHeader32 {
    pub signature: u32,
    pub file_header: ImageFileHeader,
    pub optional_header: ImageOptionalHeader32,
}

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageNtHeader64 {
    pub signature: u32,
    pub file_header: ImageFileHeader,
    pub optional_header: ImageOptionalHeader64,
}

pub const IMAGE_SIZEOF_SHORT_NAME: usize = 8;

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageSectionHeader {
    pub name: [u8; IMAGE_SIZEOF_SHORT_NAME],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

pub const IMAGE_SIZEOF_RELOCATION: usize = 10;

pub const IMAGE_REL_I386_ABSOLUTE: u16 = 0x0000;
pub const IMAGE_REL_I386_DIR16: u16    = 0x0001;
pub const IMAGE_REL_I386_REL16: u16    = 0x0002;
pub const IMAGE_REL_I386_DIR32: u16    = 0x0006;
pub const IMAGE_REL_I386_DIR32NB: u16  = 0x0007;
pub const IMAGE_REL_I386_SEG12: u16    = 0x0009;
pub const IMAGE_REL_I386_SECTION: u16  = 0x000A;
pub const IMAGE_REL_I386_SECREL: u16   = 0x000B;
pub const IMAGE_REL_I386_REL32: u16    = 0x0014;

pub const IMAGE_REL_AMD64_ABSOLUTE: u16 = 0x0000;
pub const IMAGE_REL_AMD64_ADDR64:   u16 = 0x0001;
pub const IMAGE_REL_AMD64_ADDR32:   u16 = 0x0002;
pub const IMAGE_REL_AMD64_ADDR32NB: u16 = 0x0003;
pub const IMAGE_REL_AMD64_REL32:    u16 = 0x0004;
pub const IMAGE_REL_AMD64_REL32_1:  u16 = 0x0005;
pub const IMAGE_REL_AMD64_REL32_2:  u16 = 0x0006;
pub const IMAGE_REL_AMD64_REL32_3:  u16 = 0x0007;
pub const IMAGE_REL_AMD64_REL32_4:  u16 = 0x0008;
pub const IMAGE_REL_AMD64_REL32_5:  u16 = 0x0009;
pub const IMAGE_REL_AMD64_SECTION:  u16 = 0x000A;
pub const IMAGE_REL_AMD64_SECREL:   u16 = 0x000B;
pub const IMAGE_REL_AMD64_SECREL7:  u16 = 0x000C;
pub const IMAGE_REL_AMD64_TOKEN:    u16 = 0x000D;
pub const IMAGE_REL_AMD64_SREL32:   u16 = 0x000E;
pub const IMAGE_REL_AMD64_PAIR:     u16 = 0x000F;
pub const IMAGE_REL_AMD64_SSPAN32:  u16 = 0x0010;

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageRelocation {
    pub virtual_address: u32,
    pub symbol_table_index: u32,
    pub r#type: u16,
}

pub const IMAGE_SIZEOF_BASE_RELOCATION: usize = 8;

pub const IMAGE_REL_BASED_ABSOLUTE:        usize = 0;
pub const IMAGE_REL_BASED_HIGH:            usize = 1;
pub const IMAGE_REL_BASED_LOW:             usize = 2;
pub const IMAGE_REL_BASED_HIGHLOW:         usize = 3;
pub const IMAGE_REL_BASED_HIGHADJ:         usize = 4;
pub const IMAGE_REL_BASED_MIPS_JMPADDR:    usize = 5;
pub const IMAGE_REL_BASED_ARM_MOV32A:      usize = 5;
pub const IMAGE_REL_BASED_ARM_MOV32T:      usize = 7;
pub const IMAGE_REL_BASED_IA64_IMM64:      usize = 9;
pub const IMAGE_REL_BASED_MIPS_JMPADDR16:  usize = 9;
pub const IMAGE_REL_BASED_DIR64:           usize = 10;

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct ImageBaseRelocation {
    pub virtual_address: u32,
    pub size_of_block: u32,
}




pub fn peloader_get_image_info (
    source_buffer: *mut c_void,
    source_size: usize
    ) -> (usize) {

    let mut current_ptr : usize = source_buffer as usize;
    let source_end = current_ptr + source_size;

    if current_ptr + size_of::<ImageDosHeader>() > source_end {
      log!("dos header length check fail\n");
      return (0);
    }
    let dos_header = unsafe {transmute::<usize, &mut ImageDosHeader>(current_ptr)};
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
      log!("dos header e_magic check fail\n");
      return (0);
    }
    current_ptr = current_ptr + dos_header.e_lfanew as usize;

    if current_ptr + size_of::<ImageNtHeader64>() > source_end {
      log!("NT header length check fail\n");
      return (0);
    }
    let nt_header = unsafe {transmute::<usize, &mut ImageNtHeader64>(current_ptr)};
    if nt_header.signature != IMAGE_PE_SIGNATURE {
      log!("NT header signature check fail\n");
      return (0);
    }
    if nt_header.file_header.machine != IMAGE_FILE_MACHINE_X64 {
      log!("NT header machine check fail\n");
      return (0);
    }
    if nt_header.optional_header.magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
      log!("NT header magic check fail\n");
      return (0);
    }

    return nt_header.optional_header.size_of_image as usize;
}

fn relocate_image (
    dest_buffer: *mut c_void,
    dest_size: usize,
    dest_rel_buffer: *mut c_void,
    dest_rel_size: usize,
    image_base: usize
    ) -> Result<(), usize> {
    log!("relocate_image ...\n");

    //log!("dest_buffer {:p}\n", dest_buffer);
    //log!("dest_rel_buffer {:p}\n", dest_rel_buffer);

    let delta = dest_buffer as usize - image_base;
    if delta == 0 {
      return Ok(());
    }
    let dest_rel_end = dest_rel_buffer as usize + dest_rel_size;
    let mut rel_base : *mut c_void = dest_rel_buffer;
    loop {
      //log!("rel_base {:p}\n", rel_base);
      if rel_base as usize >= dest_rel_end {
        break;
      }
      let rel = unsafe {transmute::<*mut c_void, &mut ImageBaseRelocation>(rel_base)};
      //log!("  virtual_address 0x{:x} ", rel.virtual_address);
      //log!("  size_of_block 0x{:x}\n", rel.size_of_block);
      if rel.size_of_block == 0 {
        break;
      }

      let mut relocs : *mut u16 = (rel_base as usize + size_of::<ImageBaseRelocation>() as usize) as *mut u16;
      let count = (rel.size_of_block as usize - size_of::<ImageBaseRelocation>()) / size_of::<u16>();
      //log!("  count {}\n", count);
      for x in 0..count {
        let offset = unsafe { (*relocs as usize & 0xfffusize) + rel.virtual_address as usize };
        let rel_type = unsafe { ((*relocs as usize) >> 12) as usize};
        //log!("  offset 0x{:03x} ", offset);
        //log!("  rel_type 0x{:x}\n", rel_type);
        match rel_type {
          IMAGE_REL_BASED_ABSOLUTE => {},
          IMAGE_REL_BASED_HIGH => {
            let ptr16 = (dest_buffer as usize + offset as usize) as *mut u16;
            unsafe {
              *ptr16 = *ptr16 + ((delta as u32) >> 16) as u16
            }
          },
          IMAGE_REL_BASED_LOW => {
            let ptr16 = (dest_buffer as usize + offset as usize) as *mut u16;
            unsafe {
              *ptr16 = *ptr16 + delta as u16
            }
          },
          IMAGE_REL_BASED_HIGHLOW => {
            let ptr32 = (dest_buffer as usize + offset as usize) as *mut u32;
            unsafe {
              *ptr32 = *ptr32 + delta as u32
            }
          },
          IMAGE_REL_BASED_DIR64 => {
            let ptr64 = (dest_buffer as usize + offset as usize) as *mut u64;
            unsafe {
              *ptr64 = *ptr64 + delta as u64
            }
          },
          _ => {
            log!("unknown rel_type {}\n", rel_type);
            return Err((0));
          },
        }
        relocs = (relocs as usize + size_of::<u16>()) as *mut u16;
      }
      rel_base = relocs as *mut c_void;
    }

    return Ok(());
}

/* load and relocate image */
pub fn peloader_load_image (
    dest_buffer: *mut c_void,
    dest_size: usize,
    source_buffer: *mut c_void,
    source_size: usize
    ) -> (usize) {
    log!("EFI_STUB - peloader_load_image ...\n");
    pe_dumper(source_buffer,source_size);
    let source_dos_header = unsafe {transmute::<*mut c_void, &mut ImageDosHeader>(source_buffer)};
    let pecoff_header_offset = source_dos_header.e_lfanew;

    let nt_header = unsafe {transmute::<usize, &mut ImageNtHeader64>(source_buffer as usize + pecoff_header_offset as usize)};
    let size_of_headers = nt_header.optional_header.size_of_headers;
    let size_of_optional_header = nt_header.file_header.size_of_optional_header;
    let number_of_sections = nt_header.file_header.number_of_sections;

    let first_section_offset = pecoff_header_offset as usize + offset_of!(ImageNtHeader64, optional_header) as usize + size_of_optional_header as usize;

    let total_header_size = first_section_offset + number_of_sections as usize * size_of::<ImageSectionHeader>();
    unsafe {core::ptr::copy_nonoverlapping (source_buffer, dest_buffer, total_header_size);}
    //log!("copy - {:p} => {:p} 0x{:x}\n", source_buffer, dest_buffer, total_header_size);

    for section_index in 0..number_of_sections {
      let section = unsafe {transmute::<usize, &mut ImageSectionHeader>(
                              source_buffer as usize + first_section_offset as usize + section_index as usize * size_of::<ImageSectionHeader>())};
      unsafe {
          core::ptr::write_bytes (
          (dest_buffer as usize + section.virtual_address as usize + section.virtual_size as usize) as *mut c_void,
          0,
          section.virtual_size as usize
          );
      }
      unsafe {
        core::ptr::copy_nonoverlapping (
          (source_buffer as usize + section.pointer_to_raw_data as usize) as *mut c_void,
          (dest_buffer as usize + section.virtual_address as usize) as *mut c_void,
          section.size_of_raw_data as usize
          );
      }
      //log!("copy - {:p} => {:p} 0x{:x}\n",
      //  (source_buffer as usize + section.pointer_to_raw_data as usize) as *mut c_void,
      //  (dest_buffer as usize + section.virtual_address as usize) as *mut c_void,
      //  section.size_of_raw_data as usize
      //  );
    }

    match relocate_image (
      dest_buffer,
      dest_size,
      (dest_buffer as usize + nt_header.optional_header.data_directory[IMAGE_DIRECTORY_ENTRY_BASERELOC].virtual_address as usize) as *mut c_void,
      nt_header.optional_header.data_directory[IMAGE_DIRECTORY_ENTRY_BASERELOC].size as usize,
      nt_header.optional_header.image_base as usize
      ) {
      Ok(_) => {},
      Err(_) => {return (0);},
    }

    log!("dest_buffer: {:?} nt_header->address_of_entry_point: {}\n", dest_buffer, nt_header.optional_header.address_of_entry_point);
    (dest_buffer as usize + nt_header.optional_header.address_of_entry_point as usize)
}

pub fn pe_dumper(
  buffer: *mut c_void, size: usize){
    pe_dumper_header(buffer);

}

pub fn pe_dumper_header(
  start_address: *mut c_void){
    let source_dos_header = unsafe {transmute::<*mut c_void, &mut ImageDosHeader>(start_address)};
    log!("DOS Header\n");
    log!("\t Magic Number:\t\t\t\t{:x}", source_dos_header.e_magic);
    log!("\t PE header offset:\t\t\t\t{:x}", source_dos_header.e_lfanew);

    let pecoff_header_offset = source_dos_header.e_lfanew;
    let nt_header: *mut ImageNtHeader64 = unsafe {transmute::<usize, &mut ImageNtHeader64>(start_address as usize + pecoff_header_offset as usize)};
    let nt_header_ref: ImageNtHeader64 = unsafe{*nt_header};
    log!("\nPE Signature:\n \t\t\t\t {:x}", nt_header_ref.signature);
    log!("\nFile Type: ");
    log!("\nFILE HEADER VALUES");
    log!("\n\t\t\t\t{:x} \t Machine ", nt_header_ref.file_header.machine);
    log!("\n\t\t\t\t{:x} \t NumberOfSections ", nt_header_ref.file_header.number_of_sections);
    log!("\n\t\t\t\t{:x} \t TimeDateStamp ", nt_header_ref.file_header.time_date_stamp);
    log!("\n\t\t\t\t{:x} \t PointerToSymbolTable ", nt_header_ref.file_header.pointer_to_symbol_table);
    log!("\n\t\t\t\t{:x} \t NumberOfSymbols ", nt_header_ref.file_header.number_of_symbols);
    log!("\n\t\t\t\t{:x} \t SizeOfOptionalHeader ", nt_header_ref.file_header.size_of_optional_header);
    log!("\n\t\t\t\t{:x} \t Characteristics ", nt_header_ref.file_header.characteristics);

    log!("\nOPTIONAL HEADER VALUES");
    log!("\n\t\t\t\t{:x} \t Magic  ", nt_header_ref.optional_header.magic);
    log!("\n\t\t\t\t{:x} \t MajorLinkerVersion   ", nt_header_ref.optional_header.major_linker_version);
    log!("\n\t\t\t\t{:x} \t MinorLinkerVersion   ", nt_header_ref.optional_header.minor_linker_version);
    log!("\n\t\t\t\t{:x} \t SizeOfCode   ", nt_header_ref.optional_header.size_of_code);
    log!("\n\t\t\t\t{:x} \t SizeOfInitializedData   ", nt_header_ref.optional_header.size_of_initialized_data);
    log!("\n\t\t\t\t{:x} \t SizeOfUninitializedData   ", nt_header_ref.optional_header.size_of_uninitialized_data);
    log!("\n\t\t\t\t{:x} \t AddressOfEntryPoint   ", nt_header_ref.optional_header.address_of_entry_point);
    log!("\n\t\t\t\t{:x} \t BaseOfCode    ", nt_header_ref.optional_header.base_of_code);

    log!("\n");
}