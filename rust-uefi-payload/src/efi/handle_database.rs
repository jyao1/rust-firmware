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
use core::option::Option;

use crate::efi::peloader::*;

const HANDLE_SIGNATURE: u32 = 0x4C444849; // 'I','H','D','L'

#[repr(C)]
#[derive(Debug, Clone)]
struct ProtocolStruct {
    guid : Guid,
    interface : usize,
}

impl Default for ProtocolStruct {
    fn default() -> ProtocolStruct {
      ProtocolStruct {
        guid: Guid::from_fields(0x00000000, 0x0000, 0x0000, 0x00, 0x00, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        interface: 0,
      }
    }
}

const MAX_PROTOCOL_STRUCT: usize = 16;

#[repr(C)]
#[derive(Debug, Default, Clone)]
struct ProtocolHandle {
    signature: u32,
    protocol_count: usize,
    protocol_struct : [ProtocolStruct; MAX_PROTOCOL_STRUCT],
}

const MAX_HANDLE_STRUCT: usize = 16;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct HandleDatabase {
    protocol_handle_count: usize,
    protocol_handle : [ProtocolHandle; MAX_HANDLE_STRUCT],
}

impl HandleDatabase {
    pub fn install_protocol (
        &mut self,
        handle: Handle,
        guid : *mut Guid,
        interface : *mut c_void,
    ) -> (Status, Handle) {
        let (status, mut cur_handle) = self.get_handle (handle);
        match status {
          Status::SUCCESS => {},
          Status::NOT_FOUND => {
            let (status, new_handle) = self.get_new_handle ();
            match status {
              Status::SUCCESS => {},
              _ => {return (status, core::ptr::null_mut());},
            }
            cur_handle = new_handle;
          },
          _ => {return (status, core::ptr::null_mut());},
        }
        assert!(cur_handle != core::ptr::null_mut());
        let protocol_handle = unsafe {transmute::<Handle, *mut ProtocolHandle>(cur_handle)};
        unsafe { assert!((*protocol_handle).signature == HANDLE_SIGNATURE); }

        let (status, mut cur_protocol_struct) = self.get_protocol (protocol_handle, guid);
        match status {
          Status::SUCCESS => {return (Status::INVALID_PARAMETER, core::ptr::null_mut())},
          Status::NOT_FOUND => {
            let (status, new_protocol_struct) = self.get_new_protocol (protocol_handle);
            match status {
              Status::SUCCESS => {},
              _ => {return (status, core::ptr::null_mut());},
            }
            cur_protocol_struct = new_protocol_struct;
          },
          _ => {return (status, core::ptr::null_mut());},
        }

        let protocol_struct = unsafe {transmute::<*mut ProtocolStruct, &mut ProtocolStruct>(cur_protocol_struct)};
        protocol_struct.guid = unsafe {*guid};
        protocol_struct.interface = interface as usize;

        (Status::SUCCESS, cur_handle)
    }

    pub fn install_multiple_protocol (
        &mut self,
        handle: Handle,
        count : usize,
        pair : *mut [(*mut Guid, *mut c_void); 8],
    ) -> (Status, Handle) {
        assert!(count <= 8);
        assert!(count <= MAX_PROTOCOL_STRUCT);
        assert!(count > 0);

        let (status, mut cur_handle) = self.get_handle (handle);
        match status {
          Status::SUCCESS => {
            assert!(cur_handle != core::ptr::null_mut());
            let protocol_handle = unsafe {transmute::<Handle, *mut ProtocolHandle>(cur_handle)};
            unsafe { assert!((*protocol_handle).signature == HANDLE_SIGNATURE); }
            if unsafe {(*protocol_handle).protocol_count} > MAX_PROTOCOL_STRUCT - count {
              return (Status::OUT_OF_RESOURCES, core::ptr::null_mut());
            }
            for index in 0 .. count {
              let (status, protocol_struct) = self.get_protocol (protocol_handle, unsafe {(*pair)[index].0});
              if status == Status::SUCCESS {
                return (Status::INVALID_PARAMETER, core::ptr::null_mut());
              }
            }
          },
          Status::NOT_FOUND => {
            let (status, new_handle) = self.get_new_handle ();
            match status {
              Status::SUCCESS => {},
              _ => {return (status, core::ptr::null_mut());},
            }
            cur_handle = new_handle;
          },
          _ => {return (status, core::ptr::null_mut());},
        }

        assert!(cur_handle != core::ptr::null_mut());
        let protocol_handle = unsafe {transmute::<Handle, *mut ProtocolHandle>(cur_handle)};
        unsafe { assert!((*protocol_handle).signature == HANDLE_SIGNATURE); }
        for index in 0 .. count {
            let (status, new_protocol_struct) = self.get_new_protocol (protocol_handle);
            assert!(status == Status::SUCCESS);
            let cur_protocol_struct = new_protocol_struct;

            let protocol_struct = unsafe {transmute::<*mut ProtocolStruct, &mut ProtocolStruct>(cur_protocol_struct)};
            protocol_struct.guid = unsafe {*((*pair)[index].0)};
            protocol_struct.interface = unsafe {(*pair)[index].1} as usize;
        }

        (Status::SUCCESS, cur_handle)
    }

    fn locate_handle_count (
        &mut self,
        guid : *mut Guid,
    ) -> (Status, usize) {
        let mut count = 0usize;
        for index in 0 .. self.protocol_handle_count {
          let protocol_handle = &mut self.protocol_handle[index] as *mut ProtocolHandle;
          let (status, protocol_struct) = self.get_protocol (protocol_handle, guid);
          if status == Status::SUCCESS {
            count = count + 1;
          }
        }

        if count == 0 {
          return (Status::NOT_FOUND, 0);
        }
        (Status::SUCCESS, count)
    }

    fn locate_handle_copy (
        &mut self,
        guid : *mut Guid,
        handle_buffer : *mut c_void,
        handle_buffer_size : usize
    ) {
        let address = handle_buffer as usize;
        let handle_buffer = address as *mut [Handle; MAX_HANDLE_STRUCT];
        
        let mut count = 0usize;
        for index in 0 .. self.protocol_handle_count {
          let protocol_handle = &mut self.protocol_handle[index] as *mut ProtocolHandle;
          let (status, protocol_struct) = self.get_protocol (protocol_handle, guid);
          if status == Status::SUCCESS {
            assert!(handle_buffer_size >= (count + 1) * core::mem::size_of::<Handle>());
            unsafe {(*handle_buffer)[count] = protocol_handle as Handle;}
            count = count + 1;
          }
        }
    }

    pub fn locate_handle_buffer (
        &mut self,
        guid : *mut Guid,
    ) -> (Status, usize, *mut Handle) {
        let (status, count) = self.locate_handle_count (guid);
        if status != Status::SUCCESS {
          return (status, 0, core::ptr::null_mut())
        }
        
        let mut handle_buffer_address: *mut c_void = core::ptr::null_mut();
        let status = crate::efi::allocate_pool (
                       MemoryType::BootServicesData,
                       count * size_of::<Handle>() as usize,
                       &mut handle_buffer_address);
        if status != Status::SUCCESS {
          log!("locate_handle_buffer - fail on allocate pool\n");
          return (status, 0, core::ptr::null_mut())
        }

        self.locate_handle_copy (guid, handle_buffer_address, count * core::mem::size_of::<Handle>());

        (Status::SUCCESS, count, handle_buffer_address as *mut Handle)
    }

    pub fn locate_handle (
        &mut self,
        guid : *mut Guid,
        buffer_size: usize,
        buffer: *mut Handle,
    ) -> (Status, usize) {
        let (status, count) = self.locate_handle_count (guid);
        if status != Status::SUCCESS {
          return (status, 0)
        }
        let real_size = count * core::mem::size_of::<Handle>();

        if buffer_size < real_size {
          return (Status::BUFFER_TOO_SMALL, real_size)
        }

        self.locate_handle_copy (guid, buffer as *mut c_void, real_size);

        (Status::SUCCESS, real_size)
    }

    pub fn handle_protocol (
        &mut self,
        handle: Handle,
        guid: *mut Guid,
        ) -> (Status, *mut c_void) {

        let protocol_handle : *mut ProtocolHandle = handle as *mut ProtocolHandle;
        if unsafe {(*protocol_handle).signature} != HANDLE_SIGNATURE {
          return (Status::INVALID_PARAMETER, core::ptr::null_mut());
        }
        let (status, protocol_struct) = self.get_protocol (protocol_handle, guid);
        if status != Status::SUCCESS {
          return (status, core::ptr::null_mut());
        }

        unsafe { (Status::SUCCESS, (*protocol_struct).interface as *mut c_void) }
    }

    pub fn locate_protocol (
        &mut self,
        guid: *mut Guid,
        ) -> (Status, *mut c_void) {

        for index in 0 .. self.protocol_handle_count {
          let protocol_handle = &mut self.protocol_handle[index] as *mut ProtocolHandle;
          let (status, protocol_struct) = self.get_protocol (protocol_handle, guid);
          if status == Status::SUCCESS {
            let interface : *mut c_void = unsafe { (*protocol_struct).interface } as *mut c_void;
            return (Status::SUCCESS, interface);
          }
        }

        (Status::NOT_FOUND, core::ptr::null_mut())
    }

    fn get_new_protocol (
        &mut self,
        protocol_handle : *mut ProtocolHandle
        ) -> (Status, *mut ProtocolStruct) {
        unsafe {
          if ((*protocol_handle).protocol_count >= MAX_PROTOCOL_STRUCT) {
            return (Status::OUT_OF_RESOURCES, core::ptr::null_mut());
          }
          let protocol_struct = &mut (*protocol_handle).protocol_struct[(*protocol_handle).protocol_count];
          (*protocol_handle).protocol_count = (*protocol_handle).protocol_count + 1;

          protocol_struct.guid = Guid::from_fields(0x00000000, 0x0000, 0x0000, 0x00, 0x00, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
          protocol_struct.interface = 0;

          (Status::SUCCESS, protocol_struct as *mut ProtocolStruct)
        }
    }
    
    fn get_protocol (
        &mut self,
        protocol_handle : *mut ProtocolHandle,
        guid : *mut Guid,
        ) -> (Status, *mut ProtocolStruct) {
        unsafe {
          assert!((*protocol_handle).signature == HANDLE_SIGNATURE);
          for index in 0 .. (*protocol_handle).protocol_count {
            let mut guid_data = (*protocol_handle).protocol_struct[index].guid;
            if *guid == guid_data {
              return (Status::SUCCESS, &mut (*protocol_handle).protocol_struct[index]);
            }
          }
        }
        (Status::NOT_FOUND, core::ptr::null_mut())
    }

    fn get_new_handle (
        &mut self
        ) -> (Status, Handle) {
        if (self.protocol_handle_count >= MAX_HANDLE_STRUCT) {
          return (Status::OUT_OF_RESOURCES, core::ptr::null_mut());
        }
        let protocol_handle = &mut self.protocol_handle[self.protocol_handle_count];
        self.protocol_handle_count = self.protocol_handle_count + 1;

        protocol_handle.signature = HANDLE_SIGNATURE;
        protocol_handle.protocol_count = 0;

        (Status::SUCCESS, protocol_handle as *mut ProtocolHandle as Handle)
    }

    fn get_handle (
        &mut self,
        handle : Handle,
    ) -> (Status, Handle) {
        if handle == core::ptr::null_mut() {
          return (Status::NOT_FOUND, core::ptr::null_mut());
        }
    
        let protocol_handle = unsafe {transmute::<Handle, &mut ProtocolHandle>(handle)};
        if protocol_handle.signature != HANDLE_SIGNATURE {
          return (Status::INVALID_PARAMETER, core::ptr::null_mut())
        }

        return (Status::SUCCESS, handle);
    }

    pub fn new() -> HandleDatabase {
        HandleDatabase {
            protocol_handle_count: 0,
            ..HandleDatabase::default()
        }
        
    }
}

