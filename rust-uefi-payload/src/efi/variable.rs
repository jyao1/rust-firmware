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

use r_efi::efi;
use r_efi::efi::{
    AllocateType, Boolean, CapsuleHeader, Char16, Event, EventNotify, Guid, Handle, InterfaceType,
    LocateSearchType, MemoryDescriptor, MemoryType, OpenProtocolInformationEntry, PhysicalAddress,
    ResetType, Status, Time, TimeCapabilities, TimerDelay, Tpl, MEMORY_WB
};

use core::ffi::c_void;
use core::mem::transmute;
use core::mem::size_of;

pub const GLOBAL_VARIABLE_GUID: Guid = Guid::from_fields(
    0x8BE4DF61, 0x93CA, 0x11D2, 0xAA, 0x0D, &[0x00, 0xE0, 0x98, 0x03, 0x2B, 0x8C]
);

pub const MAX_VARIABLE_NAME: usize = 32;

pub const MAX_VARIABLE_DATA: usize = 128;

#[repr(C)]
#[derive(Copy, Clone)]
struct VariableItem {
    name: [u8; MAX_VARIABLE_NAME],
    guid: [u8; 16],
    attributes: u32,
    data_size: usize,
    data: [u8; MAX_VARIABLE_DATA],
}

impl Default for VariableItem {
  fn default() -> VariableItem {
    VariableItem {
      name: [0; MAX_VARIABLE_NAME],
      guid: [0; 16],
      attributes: 0,
      data_size: 0,
      data: [0; MAX_VARIABLE_DATA],
    }
  }
}

const MAX_VARIABLE_ITEM: usize = 64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Variable {
    variable_count: usize,
    varialbe_item : [VariableItem; MAX_VARIABLE_ITEM],
}

impl Default for Variable {
  fn default() -> Variable {
    Variable {
      variable_count: 0,
      varialbe_item: [VariableItem{
                        name: [0; MAX_VARIABLE_NAME],
                        guid: [0; 16],
                        attributes: 0,
                        data_size: 0,
                        data: [0; MAX_VARIABLE_DATA],
                      } ; MAX_VARIABLE_ITEM],
    }
  }
}

impl Variable {
    pub fn get_variable (
        &mut self,
        var_name: *mut [u8; MAX_VARIABLE_NAME],
        var_guid: *mut [u8; 16],
    ) -> (Status, u32, usize, *mut [u8; MAX_VARIABLE_DATA]) {

        match self.find_variable (var_name, var_guid) {
          Some(var_item) => {
            unsafe {
              return (Status::SUCCESS, (*var_item).attributes, (*var_item).data_size, &mut (*var_item).data as *mut [u8; MAX_VARIABLE_DATA]);
            }
          },
          None => { return (Status::NOT_FOUND, 0, 0, core::ptr::null_mut()) },
        }
    }

    pub fn set_variable (
        &mut self,
        var_name: *mut [u8; MAX_VARIABLE_NAME],
        var_guid: *mut [u8; 16],
        attributes: u32,
        size: usize,
        data: *mut [u8; MAX_VARIABLE_DATA],
    ) -> (Status) {
        assert!(size <= MAX_VARIABLE_DATA);
        match self.find_variable (var_name, var_guid) {
          Some(var_item) => {
            if attributes == 0 || size == 0 || data == core::ptr::null_mut() {
              // delete the variable
              match self.find_last_variable () {
                Some(last_var_item) => {
                  unsafe { *var_item = *last_var_item; }
                },
                None => {},
              }
              self.variable_count = self.variable_count - 1;
              return (Status::SUCCESS);
            } else {
              // update this variable.
              unsafe {
                (*var_item).attributes = attributes;
                (*var_item).data_size = size;
                (*var_item).data = *data;
              }
              return (Status::SUCCESS);
            }
          },
          None => {
            if attributes == 0 || size == 0 || data == core::ptr::null_mut() {
              return (Status::SUCCESS);
            } else {
              // add this variable.
              match self.find_new_variable () {
                Some(new_var_item) => {
                  unsafe {
                    (*new_var_item).name = *var_name;
                    (*new_var_item).guid = *var_guid;
                    (*new_var_item).attributes = attributes;
                    (*new_var_item).data_size = size;
                    (*new_var_item).data = *data;
                  }
                },
                None => {},
              }
              return (Status::SUCCESS);
            }
          },
        }

        (Status::NOT_FOUND)
    }

    fn find_variable (
        &mut self,
        var_name: *mut [u8; MAX_VARIABLE_NAME],
        var_guid: *mut [u8; 16],
    ) -> Option<*mut VariableItem> {
        for var_index in 0 .. self.variable_count {
          let var_item : &mut VariableItem = &mut self.varialbe_item[var_index] as &mut VariableItem;
          if unsafe {*var_name} == var_item.name && unsafe {*var_guid} == var_item.guid {
            return Some(var_item as *mut VariableItem);
          }
        }
        None
    }
    fn find_last_variable (
        &mut self,
    ) -> Option<*mut VariableItem> {
        if self.variable_count == 0 {
          let var_item : &mut VariableItem = &mut self.varialbe_item[self.variable_count - 1] as &mut VariableItem;
          Some(var_item as *mut VariableItem)
        } else {
          None
        }
    }
    fn find_new_variable (
        &mut self,
    ) -> Option<*mut VariableItem> {
        if self.variable_count < MAX_VARIABLE_ITEM {
          let var_item : &mut VariableItem = &mut self.varialbe_item[self.variable_count] as &mut VariableItem;
          self.variable_count = self.variable_count + 1;
          Some(var_item as *mut VariableItem)
        } else {
          None
        }
    }

    pub fn new() -> Variable {
        Variable {
            ..Variable::default()
        }
    }
}

