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

use core::mem::transmute;
use core::mem::size_of;
use core::ffi::c_void;
use r_efi::efi::{Guid};

use crate::pi::hob::*;
use crate::pi::fv::*;

#[cfg(not(test))]
fn get_image_from_sections(sections_address: usize, sections_length: usize, section_type: SectionType) -> (*const c_void, usize) {
  log!("  get_image_from_sections - 0x{:x} 0x{:x}\n", sections_address, sections_length);

  let sections_end = sections_address + sections_length;

  let mut next_ptr : usize = sections_address;
  let mut current_ptr : usize;

  loop {
    current_ptr = (next_ptr +3) & 0xFFFFFFFC ;
    if current_ptr > sections_end {
      break;
    }
    log!("    checking section - 0x{:x}\n", current_ptr);
    let section_header = unsafe {transmute::<usize, &CommonSectionHeader>(current_ptr)};
    let section_size = section_header.size[0] as usize + ((section_header.size[1] as usize) << 8) + ((section_header.size[2] as usize) << 16);
    let section_header_size = size_of::<CommonSectionHeader>() as usize;
    if section_size < section_header_size {
      break;
    }
    next_ptr = current_ptr + section_size;
    if next_ptr > sections_end {
      break;
    }
    if section_header.r#type != section_type {
      continue;
    }

    let (image, size) = (current_ptr + section_header_size, section_size - section_header_size);
    log!("found image - 0x{:x} 0x{:x}\n", image, size);
    return (image as *const c_void, size);
  }
  (core::ptr::null_mut(), 0)
}

#[cfg(not(test))]
pub fn get_image_from_fv(fv_base_address: u64, fv_length: u64, fv_file_type: FvFileType, section_type: SectionType) -> (*const c_void, usize) {

  log!("get_image_from_fv - 0x{:x} 0x{:x}\n", fv_base_address, fv_length);

  let fv_header = unsafe {transmute::<usize, &FirmwareVolumeHeader>(fv_base_address as usize)};
  let fv_end = (fv_base_address + fv_length) as usize;

  assert!(fv_header.signature == FVH_SIGNATURE);

  let mut next_ptr : usize = fv_base_address as usize + fv_header.header_length as usize;
  let mut current_ptr : usize;

  loop {
    current_ptr = (next_ptr + 7) & 0xFFFFFFF8 ;
    if current_ptr > fv_end {
      break;
    }

    log!("  checking ffs - 0x{:x}\n", current_ptr);

    let ffs_header = unsafe {transmute::<usize, &FfsFileHeader>(current_ptr)};
    let ffs_size = ffs_header.size[0] as usize + ((ffs_header.size[1] as usize) << 8) + ((ffs_header.size[2] as usize) << 16);
    log!("    ffs size - 0x{:x}\n - ffs type: {:x}\n", ffs_size, ffs_header.r#type);
    let ffs_header_size = size_of::<FfsFileHeader>() as usize;
    let section_header_size = size_of::<CommonSectionHeader>() as usize;
    if ffs_size < ffs_header_size + section_header_size {
      break;
    }
    next_ptr = current_ptr + ffs_size;
    if next_ptr > fv_end {
      break;
    }
    if ffs_header.r#type != fv_file_type {
      continue;
    }

    let (image, size) = get_image_from_sections (current_ptr + ffs_header_size, ffs_size - ffs_header_size, section_type);
    if image != core::ptr::null_mut() {
      return (image, size);
    }
  }

  (core::ptr::null_mut(), 0)
}

pub fn find_image_in_fv (hob: *const c_void) -> (*const c_void, usize) {
  let mut hob_header : *const Header = hob as *const Header;

  loop {
    let header = unsafe {transmute::<*const Header, &Header>(hob_header)};
    match header.r#type {
      HOB_TYPE_FV => {
        let fv_hob = unsafe {transmute::<*const Header, &FirmwareVolume>(hob_header)};
        let (image, size) = get_image_from_fv (fv_hob.base_address, fv_hob.length, FV_FILETYPE_APPLICATION, SECTION_PE32);
        if image != core::ptr::null_mut() {
          return (image, size);
        }
      }
      HOB_TYPE_END_OF_HOB_LIST => {
        break;
      }
      _ => {}
    }
    let addr = hob_header as usize + header.length as usize;
    hob_header = addr as *const Header;
  }

  (core::ptr::null_mut(), 0)
}
