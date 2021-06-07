// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use plain::Plain;

use core::convert::TryInto;
use core::ffi::c_void;

const SIZE_4KB: u64 = 0x00001000u64;

/// Number of bytes in an identifier.
pub const SIZEOF_IDENT: usize = 16;

/// Loadable program segment
pub const PT_LOAD: u32 = 1;

// ELFMAG b"\x7FELF"
pub const ELFMAG: [u8; 4] = [127, 69, 76, 70];

/// EI_CLASS
pub const EI_CLASS: usize = 4;
/// Invalid class.
pub const ELFCLASSNONE: u8 = 0;
/// 32-bit objects.
pub const ELFCLASS32: u8 = 1;
/// 64-bit objects.
pub const ELFCLASS64: u8 = 2;

#[repr(packed)]
#[derive(Default)]
pub struct ELFHeader64 {
    /// Magic number and other info
    pub e_ident: [u8; SIZEOF_IDENT],
    /// Object file type
    pub e_type: u16,
    /// Architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

unsafe impl Plain for ELFHeader64 {}

impl ELFHeader64 {
    pub fn from_bytes(buf: &[u8]) -> &ELFHeader64 {
        plain::from_bytes(buf).unwrap()
    }
}

#[repr(packed)]
#[derive(Default)]
pub struct ELFHeader32 {
    /// Magic number and other info
    pub e_ident: [u8; SIZEOF_IDENT],
    /// Object file type
    pub e_type: u16,
    /// Architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u32,
    /// Program header table file offset
    pub e_phoff: u32,
    /// Section header table file offset
    pub e_shoff: u32,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

unsafe impl Plain for ELFHeader32 {}

impl ELFHeader32 {
    pub fn from_bytes(buf: &[u8]) -> &ELFHeader32 {
        plain::from_bytes(buf).unwrap()
    }
}

#[repr(packed)]
#[derive(Default)]
pub struct ProgramHeader32 {
    /// Segment type
    pub p_type: u32,
    /// Segment file offset
    pub p_offset: u32,
    /// Segment virtual address
    pub p_vaddr: u32,
    /// Segment physical address
    pub p_paddr: u32,
    /// Segment size in file
    pub p_filesz: u32,
    /// Segment size in memory
    pub p_memsz: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Segment alignment
    pub p_align: u32,
}

unsafe impl Plain for ProgramHeader32 {}

impl ProgramHeader32 {
    pub fn slice_from_bytes(buf: &[u8]) -> &[Self] {
        plain::slice_from_bytes(buf).unwrap()
    }
}

#[repr(packed)]
#[derive(Default)]
pub struct ProgramHeader64 {
    /// Segment type
    pub p_type: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr: u64,
    /// Segment physical address
    pub p_paddr: u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

unsafe impl Plain for ProgramHeader64 {}

impl ProgramHeader64 {
    pub fn slice_from_bytes(buf: &[u8]) -> &[Self] {
        plain::slice_from_bytes(buf).unwrap()
    }
}

/// flag  ture align to low address else high address
fn align_value(value: u64, align: u64, flag: bool) -> u64 {
    if flag {
        value & ((!(align - 1)) as u64)
    } else {
        value - (value & (align - 1)) as u64 + align
    }
}

fn relocate_elf64(image: *const c_void, data_slice: &[u8]) -> (u64, u64, u64) {
    let elf_header = ELFHeader64::from_bytes(data_slice);
    let phdr_slice = unsafe {
        core::slice::from_raw_parts(
            (image as u64 + elf_header.e_phoff as u64) as *const u8,
            (elf_header.e_ehsize * elf_header.e_phnum) as usize,
        )
    };

    let pheaders = ProgramHeader64::slice_from_bytes(phdr_slice);

    let mut bottom: u64 = 0xFFFFFFFFu64;
    let mut top: u64 = 0u64;

    for ph in pheaders.iter() {
        if ph.p_type == PT_LOAD {
            if bottom > ph.p_vaddr {
                bottom = ph.p_vaddr;
            }
            if top < ph.p_vaddr + ph.p_memsz {
                top = ph.p_vaddr + ph.p_memsz;
            }
        }
    }
    bottom = align_value(bottom, SIZE_4KB, true);
    top = align_value(top, SIZE_4KB, false);

    // load per program header
    for ph in pheaders.iter() {
        if ph.p_type == PT_LOAD {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (image as u64 + ph.p_offset as u64) as *mut u8,
                    ph.p_vaddr as *const u8 as *mut u8,
                    ph.p_filesz as usize,
                )
            };
        }
    }
    (elf_header.e_entry as u64, bottom, top - bottom)
}

fn relocate_elf32(image: *const c_void, data_slice: &[u8]) -> (u64, u64, u64) {
    let elf_header = ELFHeader32::from_bytes(data_slice);
    let phdr_slice = unsafe {
        core::slice::from_raw_parts(
            (image as u64 + elf_header.e_phoff as u64) as *const u8,
            (elf_header.e_ehsize * elf_header.e_phnum) as usize,
        )
    };

    let pheaders = ProgramHeader32::slice_from_bytes(phdr_slice);

    let mut bottom: u32 = 0xFFFFFFFFu32;
    let mut top: u32 = 0u32;

    for ph in pheaders.iter() {
        if ph.p_type == PT_LOAD {
            if bottom > ph.p_vaddr {
                bottom = ph.p_vaddr;
            }
            if top < ph.p_vaddr + ph.p_memsz {
                top = ph.p_vaddr + ph.p_memsz;
            }
        }
    }
    bottom = align_value(bottom as u64, SIZE_4KB, true) as u32;
    top = align_value(top as u64, SIZE_4KB, false) as u32;

    // load per program header
    for ph in pheaders.iter() {
        if ph.p_type == PT_LOAD {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (image as u64 + ph.p_offset as u64) as *mut u8,
                    ph.p_vaddr as *const u8 as *mut u8,
                    ph.p_filesz as usize,
                )
            };
        }
    }
    (
        elf_header.e_entry as u64,
        bottom as u64,
        (bottom - top) as u64,
    )
}

pub fn relocate_elf(image: *const c_void, size: usize) -> (u64, u64, u64) {
    // parser file and get entry point
    let data_slice = unsafe { core::slice::from_raw_parts(image as *const u8, size as usize) };

    let data_header = data_slice[0..4].try_into().unwrap();
    match data_header {
        ELFMAG => match data_slice[EI_CLASS] {
            ELFCLASS32 => relocate_elf32(image, data_slice),
            ELFCLASS64 => relocate_elf64(image, data_slice),
            _ => (0u64, 0u64, 0u64),
        },
        _ => (0u64, 0u64, 0u64),
    }
}
