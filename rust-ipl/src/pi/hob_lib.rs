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
use crate::pi::hob::*;

use r_efi::efi;
use r_efi::efi::{
    AllocateType, MemoryType, PhysicalAddress, Status,
};


#[cfg(not(test))]
fn dump_hob_header(hob_header: & Header) {
  log!("Hob:\n");
  log!("  header.type            - 0x{:x}\n", hob_header.r#type);
  log!("  header.length          - 0x{:x}\n", hob_header.length);
}

#[cfg(not(test))]
fn dump_phit_hob(phit_hob: & HandoffInfoTable) {
  log!("PhitHob:\n");
  log!("  version                - 0x{:x}\n", phit_hob.version);
  log!("  boot_mode              - 0x{:x}\n", phit_hob.boot_mode);
  log!("  efi_memory_top         - 0x{:016x}\n", phit_hob.efi_memory_top);
  log!("  efi_memory_bottom      - 0x{:016x}\n", phit_hob.efi_memory_bottom);
  log!("  efi_free_memory_top    - 0x{:016x}\n", phit_hob.efi_free_memory_top);
  log!("  efi_free_memory_bottom - 0x{:016x}\n", phit_hob.efi_free_memory_bottom);
  log!("  efi_end_of_hob_list    - 0x{:016x}\n", phit_hob.efi_end_of_hob_list);
}

#[cfg(not(test))]
fn dump_resource_hob(resource_hob: & ResourceDescription) {
  log!(
    "ResourceDescription 0x{:08x} : 0x{:016x} - 0x{:016x} (0x{:08x})\n",
    resource_hob.resource_type,
    resource_hob.physical_start,
    resource_hob.physical_start + resource_hob.resource_length - 1,
    resource_hob.resource_attribute
    );
}

#[cfg(not(test))]
fn dump_allocation_hob(allocation_hob: & MemoryAllocation) {
  log!(
    "MemoryAllocation 0x{:08x} : 0x{:016x} - 0x{:016x}\n",
    allocation_hob.alloc_descriptor.memory_type as u32,
    allocation_hob.alloc_descriptor.memory_base_address,
    allocation_hob.alloc_descriptor.memory_base_address + allocation_hob.alloc_descriptor.memory_length - 1,
    );
}

#[cfg(not(test))]
fn dump_fv_hob(fv_hob: & FirmwareVolume) {
  log!(
    "FirmwareVolume : 0x{:016x} - 0x{:016x}\n",
    fv_hob.base_address,
    fv_hob.base_address + fv_hob.length - 1
    );
}

#[cfg(not(test))]
fn dump_cpu_hob(cpu_hob: & Cpu) {
  log!(
    "Cpu : mem size {} , io size {}\n",
    cpu_hob.size_of_memory_space,
    cpu_hob.size_of_io_space
    );
}

#[cfg(not(test))]
pub fn dump_hob(hob: *const c_void) {

  let mut hob_header : *const Header = hob as *const Header;

  loop {
    let header = unsafe {transmute::<*const Header, &Header>(hob_header)};
    match header.r#type {
      HOB_TYPE_HANDOFF => {
        let phit_hob = unsafe {transmute::<*const Header, &HandoffInfoTable>(hob_header)};
        dump_phit_hob (phit_hob);
      }
      HOB_TYPE_RESOURCE_DESCRIPTOR => {
        let resource_hob = unsafe {transmute::<*const Header, &ResourceDescription>(hob_header)};
        dump_resource_hob (resource_hob);
      }
      HOB_TYPE_MEMORY_ALLOCATION => {
        let allocation_hob = unsafe {transmute::<*const Header, &MemoryAllocation>(hob_header)};
        dump_allocation_hob (allocation_hob);
      }
      HOB_TYPE_FV => {
        let fv_hob = unsafe {transmute::<*const Header, &FirmwareVolume>(hob_header)};
        dump_fv_hob (fv_hob);
      }
      HOB_TYPE_CPU => {
        let cpu_hob = unsafe {transmute::<*const Header, &Cpu>(hob_header)};
        dump_cpu_hob (cpu_hob);
      }
      HOB_TYPE_END_OF_HOB_LIST => {
        break;
      }
      _ => {
        dump_hob_header (header);
      }
    }
    let addr = hob_header as usize + header.length as usize;
    hob_header = addr as *const Header;
  }
}

#[cfg(not(test))]
pub fn get_hob_total_size(hob: *const c_void) -> usize {
  let phit = unsafe {transmute::<*const c_void, &HandoffInfoTable>(hob)};
  phit.efi_end_of_hob_list as usize - hob as usize
}
