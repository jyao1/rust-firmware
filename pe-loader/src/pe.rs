// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use scroll::{Pread, Pwrite};

const PE_SIGNATURE: u32 = 0x00004550;
const DOS_SIGNATURE: u16 = 0x5a4d;
const MACHINE_X64: u16 = 0x8664;
const OPTIONAL_HDR64_MAGIC: u16 = 0x20b;

const REL_BASED_DIR64: u8 = 10;

pub fn is_pe(pe_image: &[u8]) -> bool {
    if pe_image.len() <= 0x42 {
        return false;
    }
    if pe_image.pread::<u16>(0).unwrap() != DOS_SIGNATURE {
        return false;
    }
    let pe_header_offset = pe_image.pread::<u32>(0x3c).unwrap() as usize;

    if pe_image.len() <= pe_header_offset {
        return false
    }

    let pe_region = &pe_image[pe_header_offset..];

    if pe_region.pread::<u32>(0).unwrap() != PE_SIGNATURE {
        return false;
    }
    // if pe is x64
    if pe_region.pread::<u16>(4).unwrap() != MACHINE_X64 {
        return false;
    }
    true
}

pub fn relocate(pe_image: &[u8], new_pe_image: &mut [u8], new_image_base: usize) -> Option<usize> {
    log::info!("start relocate...");
    let image_buffer = pe_image;
    let loaded_buffer = &mut new_pe_image[..];

    let pe_header_offset = pe_image.pread::<u32>(0x3c).unwrap() as usize;
    let pe_region = &pe_image[pe_header_offset..];

    let num_sections = pe_region.pread::<u16>(6).unwrap() as usize;
    let optional_header_size = pe_region.pread::<u16>(20).unwrap() as usize;
    let optional_region = &image_buffer[24+pe_header_offset..];

    // check optional_hdr64_magic
    if optional_region.pread::<u16>(0).unwrap() != OPTIONAL_HDR64_MAGIC {
        return None;
    }

    let entry_point = optional_region.pread::<u32>(16).unwrap();
    let image_base = optional_region.pread::<u64>(24).unwrap();

    let sections_buffer = &image_buffer[(24 + pe_header_offset + optional_header_size)..];

    let total_header_size =
        (24 + pe_header_offset + optional_header_size + num_sections * 40) as usize;
    loaded_buffer[0..total_header_size].copy_from_slice(&image_buffer[0..total_header_size]);
    let _ = loaded_buffer.pwrite(new_image_base as u64, (24 + pe_header_offset + 24) as usize);

    let sections = Sections::parse(sections_buffer, num_sections as usize).unwrap();
    // Load the PE header into the destination memory
    for section in sections {
        let section_size = core::cmp::min(section.size_of_raw_data, section.virtual_size);
        let section_range =
            section.virtual_address as usize..(section.virtual_address + section_size) as usize;
        loaded_buffer[section_range.clone()].fill(0);
        loaded_buffer[section_range.clone()].copy_from_slice(
            &image_buffer[section.pointer_to_raw_data as usize
                ..(section.pointer_to_raw_data + section_size) as usize],
        );
    }

    let sections = Sections::parse(sections_buffer, num_sections as usize).unwrap();
    for section in sections {
        if &section.name[0..6] == b".reloc" {
            reloc_to_base(
                loaded_buffer,
                image_buffer,
                &section,
                image_base as usize,
                new_image_base as usize,
            );
        }
    }

    Some(new_image_base + entry_point as usize)
}

pub fn relocate_pe_mem(image: &[u8], loaded_buffer: &mut [u8]) -> (u64, u64, u64) {
    // parser file and get entry point
    let image_buffer = image;
    let image_size = image.len();
    let new_image_base = loaded_buffer as *const [u8] as *const u8 as usize;

    let res = relocate(image_buffer, loaded_buffer, new_image_base).unwrap();

    (
        res as u64,
        new_image_base as usize as u64,
        image_size as u64,
    )
}

#[derive(Debug, Default, Pread, Pwrite)]
pub struct Section {
    name: [u8; 8], // 8
    virtual_size: u32, //4
    virtual_address: u32, //4
    size_of_raw_data: u32,//4
    pointer_to_raw_data: u32,//4
    pointer_to_relocations: u32,//4
    pointer_to_line_numbers: u32,//4
    number_of_relocations: u16,//2
    number_of_line_numbers: u16,//2
    characteristics: u32,//4
}

pub struct Sections<'a> {
    index: usize,
    entries: &'a [u8],
    num_sections: usize
}

impl<'a> Sections<'a> {
    // section entries byties, num_sections: total sections
    pub fn parse(entries: &'a [u8], num_sections: usize) -> Option<Self> {
        Some(Sections {
            index: 0,
            entries,
            num_sections,
        })
    }
}

impl<'a> Iterator for Sections<'a> {
    type Item = Section;
    fn next(&mut self) -> Option<Self::Item> {
        const ENTRY_SIZE: usize = 40;
        if self.index == self.num_sections {
            return None;
        }
        let offset = self.index * ENTRY_SIZE;

        let current_bytes = &self.entries[offset..];

        let section: Section = current_bytes.pread(0).unwrap();

        self.index += 1;
        Some(section)
    }
}

#[derive(Clone, Copy)]
pub struct RelocationEntry {
    pub entry_type: u8,
    pub offset: u32,
}

pub struct RelocationEntries<'a> {
    index: usize,
    entries: &'a [u8],
}

impl<'a> RelocationEntries<'a> {
    pub fn parse(entries: &'a [u8]) -> Option<Self> {
        Some(RelocationEntries { index: 0, entries })
    }
}

impl<'a> Iterator for RelocationEntries<'a> {
    type Item = RelocationEntry;
    fn next(&mut self) -> Option<Self::Item> {
        const ENTRY_SIZE: usize = 2;
        if self.index * core::mem::size_of::<u16>() > self.entries.len() - ENTRY_SIZE {
            return None;
        }

        let entry: u16 = self.entries.pread(self.index * ENTRY_SIZE).ok()?;
        let entry_type = (entry >> 12) as u8;
        let entry_offset = (entry & 0xfff) as u32;

        let res = RelocationEntry {
            entry_type,
            offset: entry_offset,
        };
        self.index += 1;
        Some(res)
    }
}

pub struct Relocations<'a> {
    offset: usize,
    relocations: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct Relocation<'a> {
    pub page_rva: u32,
    pub block_size: u32,
    pub entries: &'a [u8],
}

impl<'a> Relocations<'a> {
    pub fn parse(bytes: &'a [u8]) -> Option<Self> {
        Some(Relocations {
            offset: 0,
            relocations: bytes,
        })
    }
}

impl<'a> Iterator for Relocations<'a> {
    type Item = Relocation<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.relocations.len() - 8 {
            return None;
        }
        let bytes = &self.relocations[self.offset..];
        let page_rva = bytes.pread(0).ok()?;
        let block_size = bytes.pread(core::mem::size_of::<u32>()).ok()?;
        let entries = &bytes[(core::mem::size_of::<u32>() * 2) as usize..block_size as usize];
        let res = Relocation {
            page_rva,
            block_size,
            entries,
        };
        self.offset += block_size as usize;
        Some(res)
    }
}

fn reloc_to_base(
    loaded_buffer: &mut [u8],
    image_buffer: &[u8],
    section: &Section,
    image_base: usize,
    new_image_base: usize,
) {
    let section_size = core::cmp::min(section.size_of_raw_data, section.virtual_size);

    let relocation_range_in_image =
        section.pointer_to_raw_data as usize..(section.pointer_to_raw_data + section_size) as usize;

    let relocations = Relocations::parse(&image_buffer[relocation_range_in_image]).unwrap();
    for relocation in relocations {
        for entry in RelocationEntries::parse(relocation.entries).unwrap() {
            match entry.entry_type {
                REL_BASED_DIR64 => {
                    let location = (relocation.page_rva + entry.offset) as usize;
                    let value: u64 = loaded_buffer.pread(location).unwrap();
                    let _ = loaded_buffer.pwrite(
                        value - image_base as u64 + new_image_base as u64,
                        location as usize,
                    );
                }
                _ => continue,
            }
        }
    }
}