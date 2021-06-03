// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use scroll::Pwrite;

const SIZE_4KB: u64 = 0x00001000u64;

/// Number of bytes in an identifier.
pub const SIZEOF_IDENT: usize = 16;

/// Loadable program segment
pub const PT_LOAD: u32 = 1;

pub const R_X86_64_RELATIVE: u32 = 8;

// ELFMAG b"\x7FELF"
pub const ELFMAG: [u8; 4] = [127, 69, 76, 70];

pub fn is_elf(image: &[u8]) -> bool {
    goblin::elf::Elf::parse(image).is_ok()
}

pub fn relocate_elf(image: &[u8], loaded_buffer: &mut [u8]) -> (u64, u64, u64) {
    let new_image_base = loaded_buffer as *const [u8] as *const u8 as usize;
    // parser file and get entry point
    let elf = goblin::elf::Elf::parse(image).unwrap();

    let mut bottom: u64 = 0xFFFFFFFFu64;
    let mut top: u64 = 0u64;

    for ph in elf.program_headers.as_slice() {
        if ph.p_type == PT_LOAD {
            if bottom > ph.p_vaddr {
                bottom = ph.p_vaddr;
            }
            if top < ph.p_vaddr + ph.p_memsz {
                top = ph.p_vaddr + ph.p_memsz;
            }
        }
    }
    let mut bottom = bottom + new_image_base as u64;
    let mut top = top + new_image_base as u64;
    bottom = align_value(bottom, SIZE_4KB, true);
    top = align_value(top, SIZE_4KB, false);

    // load per program header
    for ph in elf.program_headers.as_slice() {
        if ph.p_type == PT_LOAD && ph.p_memsz != 0 {
            let data_range = ph.p_offset as usize..(ph.p_offset + ph.p_filesz) as usize;
            let loaded_range = (ph.p_vaddr) as usize..(ph.p_vaddr + ph.p_filesz) as usize;
            loaded_buffer[loaded_range].copy_from_slice(&image[data_range]);
        }
    }

    // relocate to base
    for reloc in elf.dynrelas.into_iter() {
        if reloc.r_type == R_X86_64_RELATIVE {
            let r_addend = reloc.r_addend;
            if r_addend.is_none() {
                continue;
            }
            let r_addend = r_addend.unwrap();
            loaded_buffer
                .pwrite::<u64>(
                    new_image_base as u64 + r_addend as u64,
                    reloc.r_offset as usize,
                )
                .unwrap();
        }
    }

    (
        elf.entry + new_image_base as u64,
        bottom as u64,
        (top - bottom) as u64,
    )
}

/// flag  ture align to low address else high address
fn align_value(value: u64, align: u64, flag: bool) -> u64 {
    if flag {
        value & ((!(align - 1)) as u64)
    } else {
        value - (value & (align - 1)) as u64 + align
    }
}
