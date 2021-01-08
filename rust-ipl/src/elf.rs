// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use plain::Plain;

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
#[derive(Default, Debug)]
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

unsafe impl Plain for ELFHeader64 {
}

impl ELFHeader64 {
    pub fn from_bytes(buf: &[u8]) -> &ELFHeader64 {
        plain::from_bytes(buf).unwrap()
    }
}

#[repr(packed)]
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
pub struct ProgramHeader64 {
    /// Segment type
    pub p_type  : u32,
    /// Segment flags
    pub p_flags : u32,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr : u64,
    /// Segment physical address
    pub p_paddr : u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz : u64,
    /// Segment alignment
    pub p_align : u64,
}

unsafe impl Plain for ProgramHeader64 {}

impl ProgramHeader64 {
    pub fn slice_from_bytes(buf: &[u8]) -> &[Self] {
        plain::slice_from_bytes(buf).unwrap()
    }
}