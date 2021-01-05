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

use r_efi::{eficall, eficall_abi};

use r_efi::protocols::device_path::Protocol as DevicePathProtocol;
//use r_efi::protocols::loaded_image::Protocol as LoadedImageProtocol;
use crate::efi::device_path::MemoryMaped as MemoryMappedDevicePathProtocol;
use r_efi::protocols::device_path::End as EndDevicePath;

use core::ffi::c_void;
use core::mem::transmute;
use core::mem::size_of;

use crate::efi::peloader::*;

// HACK: Until r-util/r-efi#11 gets merged
#[cfg(not(test))]
#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: Handle,
    pub system_table: *mut efi::SystemTable,

    pub device_handle: Handle,
    pub file_path: *mut r_efi::protocols::device_path::Protocol,
    pub reserved: *mut core::ffi::c_void,

    pub load_options_size: u32,
    pub load_options: *mut core::ffi::c_void,

    pub image_base: *mut core::ffi::c_void,
    pub image_size: u64,
    pub image_code_type: efi::MemoryType,
    pub image_data_type: efi::MemoryType,
    pub unload: eficall! {fn(
        Handle,
    ) -> Status},
}

impl Default for LoadedImageProtocol {
  fn default() -> LoadedImageProtocol {
    LoadedImageProtocol {
      revision: r_efi::protocols::loaded_image::REVISION,
      parent_handle: core::ptr::null_mut(),
      system_table: unsafe {&mut crate::efi::ST as *mut r_efi::system::SystemTable},
      device_handle: core::ptr::null_mut(),
      file_path: core::ptr::null_mut(),
      reserved: core::ptr::null_mut(),
      load_options_size: 0,
      load_options: core::ptr::null_mut(),
      image_base: core::ptr::null_mut(),
      image_size: 0,
      image_code_type: efi::MemoryType::LoaderCode,
      image_data_type: efi::MemoryType::LoaderData,
      unload: crate::efi::image_unload,
    }
  }
}

pub const IMAGE_INFO_GUID: Guid = Guid::from_fields(
    0xdecf2644, 0xbc0, 0x4840, 0xb5, 0x99, &[0x13, 0x4b, 0xee, 0xa, 0x9e, 0x71]
);

const IMAGE_INFO_SIGNATURE: u32 = 0x49444849; // 'I','H','D','I'

#[derive(Default)]
#[repr(C)]
struct ImageInfo {
    signature: u32,
    source_buffer: usize,
    source_size: usize,
    entry_point: usize,
    loaded_image: LoadedImageProtocol,
}

#[derive(Default)]
pub struct Image {
    image_count: usize,
}

impl Image {
    pub fn load_image (
        &mut self,
        parent_image_handle: Handle,
        device_path: *mut c_void,
        source_buffer: *mut c_void,
        source_size: usize,
    ) -> (Status, Handle) {
        let mut handle_address: *mut c_void = core::ptr::null_mut();

        let device_path_size = crate::efi::device_path::get_device_path_size (device_path as *mut DevicePathProtocol);

        let status = crate::efi::allocate_pool (MemoryType::BootServicesData, size_of::<ImageInfo>() + device_path_size, &mut handle_address);
        if status != Status::SUCCESS {
          log!("load_image - fail on allocate pool\n");
          return (status, core::ptr::null_mut())
        }
        let device_path_buffer : *mut c_void = (handle_address as usize + size_of::<ImageInfo>()) as *mut c_void;
        unsafe {core::ptr::copy_nonoverlapping (device_path, device_path_buffer, device_path_size);}

        let handle = unsafe {transmute::<*mut c_void, &mut ImageInfo>(handle_address)};
        handle.signature = IMAGE_INFO_SIGNATURE;
        handle.source_buffer = source_buffer as usize;
        handle.source_size   = source_size;

        let image_size = peloader_get_image_info (source_buffer, source_size);
        log!("load_image - image_size 0x{:x}\n", image_size);
        if image_size == 0 {
          return (Status::SECURITY_VIOLATION, core::ptr::null_mut())
        }
        let mut image_address : *mut c_void = core::ptr::null_mut();
        let status = crate::efi::allocate_pool (MemoryType::BootServicesData, image_size, &mut image_address);
        if status != Status::SUCCESS {
          log!("load_image - fail on allocate pool\n");
          return (Status::OUT_OF_RESOURCES, core::ptr::null_mut())
        }
        log!("image_address - {:p}\n", image_address);

        handle.entry_point = peloader_load_image (image_address, image_size, source_buffer, source_size);
        log!("entry_point - 0x{:x}\n", handle.entry_point);
        if handle.entry_point == 0 {
          return (Status::SECURITY_VIOLATION, core::ptr::null_mut())
        }

        let mut image_handle : Handle = core::ptr::null_mut();
        let status = crate::efi::install_protocol_interface (
                       &mut image_handle,
                       &mut IMAGE_INFO_GUID as *mut Guid,
                       InterfaceType::NativeInterface,
                       handle_address
                       );

        let loaded_image_address: *mut c_void = (handle_address as usize + offset_of!(ImageInfo, loaded_image)) as *mut c_void;
        let mut loaded_image = unsafe {transmute::<*mut c_void, &mut LoadedImageProtocol>(loaded_image_address)};
        loaded_image.revision = r_efi::protocols::loaded_image::REVISION;
        loaded_image.parent_handle = parent_image_handle;
        loaded_image.system_table = unsafe {&mut crate::efi::ST as *mut r_efi::system::SystemTable};
        loaded_image.device_handle = core::ptr::null_mut();
        loaded_image.file_path = device_path_buffer as *mut DevicePathProtocol;
        loaded_image.reserved = core::ptr::null_mut();
        loaded_image.load_options_size = 0;
        loaded_image.load_options = core::ptr::null_mut();
        loaded_image.image_base = image_address as *mut c_void;
        loaded_image.image_size = image_size as u64;
        loaded_image.image_code_type = MemoryType::LoaderCode;
        loaded_image.image_data_type = MemoryType::BootServicesData;
        loaded_image.unload = crate::efi::image_unload;


        let status = crate::efi::install_protocol_interface (
                       &mut loaded_image.device_handle as *mut Handle,
                       &mut r_efi::protocols::device_path::PROTOCOL_GUID as *mut Guid,
                       InterfaceType::NativeInterface,
                       device_path_buffer
                       );

        let status = crate::efi::install_protocol_interface (
                       &mut image_handle,
                       &mut r_efi::protocols::loaded_image::PROTOCOL_GUID as *mut Guid,
                       InterfaceType::NativeInterface,
                       loaded_image_address
                       );

        (status, image_handle)
    }
    pub fn start_image (
        &mut self,
        image_handle: Handle,
    ) -> (Status, usize, *mut Char16) {

        let mut handle_address: *mut c_void = core::ptr::null_mut();
        let status = crate::efi::handle_protocol (
                       image_handle,
                       &mut IMAGE_INFO_GUID,
                       &mut handle_address
                       );
        if status != Status::SUCCESS {
          return (Status::INVALID_PARAMETER, 0, core::ptr::null_mut())
        }

        let handle = unsafe {transmute::<*mut c_void, &mut ImageInfo>(handle_address)};
        if handle.signature != IMAGE_INFO_SIGNATURE {
          return (Status::INVALID_PARAMETER, 0, core::ptr::null_mut())
        }

        log!("start_image - entry_point 0x{:x}\n", handle.entry_point);

        let ptr = handle.entry_point as *const ();
        let code: extern "win64" fn(Handle, *mut efi::SystemTable) -> Status = unsafe { core::mem::transmute(ptr) };

        let status = unsafe { (code)(image_handle, &mut crate::efi::ST) };

        (status, 0, core::ptr::null_mut())
    }

    pub fn new() -> Image {
        Image {
            image_count: 0,
        }
    }
}

