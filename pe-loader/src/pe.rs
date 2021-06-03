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

use goblin::pe::section_table::SectionTable;

use scroll::{Pread, Pwrite};

const REL_BASED_DIR64: u8 = 10;

pub fn is_pe(pe_image: &[u8]) -> bool {
    goblin::pe::PE::parse(pe_image).is_ok()
}

pub fn relocate(pe_image: &[u8], new_pe_image: &mut [u8], new_image_base: usize) -> Option<usize> {
    log::info!("start relocate...");
    let image_buffer = pe_image;
    let loaded_buffer = &mut new_pe_image[..];

    let pe = goblin::pe::PE::parse(image_buffer).ok()?;
    let _header = pe.header;
    let image_base = pe.image_base;
    let entry_point = pe.entry;

    let pe_header_offset = pe.header.dos_header.pe_pointer as usize;
    let num_sections = pe.header.coff_header.number_of_sections as usize;
    let optional_header_size = pe.header.coff_header.size_of_optional_header as usize;
    let total_header_size =
        (24 + pe_header_offset + optional_header_size + num_sections * 40) as usize;
    loaded_buffer[0..total_header_size].copy_from_slice(&image_buffer[0..total_header_size]);
    let _ = loaded_buffer.pwrite(new_image_base as u64, (24 + pe_header_offset + 24) as usize);

    // Load the PE header into the destination memory
    for section in pe.sections.iter() {
        let section_size = core::cmp::min(section.size_of_raw_data, section.virtual_size);
        let section_range =
            section.virtual_address as usize..(section.virtual_address + section_size) as usize;
        loaded_buffer[section_range.clone()].fill(0);
        loaded_buffer[section_range.clone()].copy_from_slice(
            &image_buffer[section.pointer_to_raw_data as usize
                ..(section.pointer_to_raw_data + section_size) as usize],
        );
    }
    for section in pe.sections.iter() {
        if &section.name[0..6] == b".reloc" {
            reloc_to_base(
                loaded_buffer,
                image_buffer,
                section,
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
    section: &SectionTable,
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
