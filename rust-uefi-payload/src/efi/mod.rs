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

mod alloc;
mod block;
mod file;
mod device_path;
mod image;
mod event;
mod handle_database;
mod variable;
mod conout;
mod conin;
mod peloader;
mod init;

use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;
use cpuio::Port;
use core::mem::transmute;

use r_efi::efi;
use r_efi::efi::{
    AllocateType, Boolean, CapsuleHeader, Char16, Event, EventNotify, Guid, Handle, InterfaceType,
    LocateSearchType, MemoryDescriptor, MemoryType, OpenProtocolInformationEntry, PhysicalAddress,
    ResetType, Status, Time, TimeCapabilities, TimerDelay, Tpl, OPEN_PROTOCOL_GET_PROTOCOL, MEMORY_WB,
};

use r_efi::protocols::simple_text_input::InputKey;
use r_efi::protocols::simple_text_input::Protocol as SimpleTextInputProtocol;
use r_efi::protocols::simple_text_input_ex::{KeyData, KeyToggleState, KeyNotifyFunction};
use r_efi::protocols::simple_text_input_ex::Protocol as SimpleTextInputExProtocol;
use r_efi::protocols::simple_text_output::Mode as SimpleTextOutputMode;
use r_efi::protocols::simple_text_output::Protocol as SimpleTextOutputProtocol;
//use r_efi::protocols::loaded_image::Protocol as LoadedImageProtocol;
use r_efi::protocols::device_path::Protocol as DevicePathProtocol;
use crate::efi::device_path::MemoryMaped as MemoryMappedDevicePathProtocol;
use r_efi::protocols::device_path::End as EndDevicePath;
use r_efi::protocols::device_path_utilities::Protocol as DevicePathUtilities;

use r_efi::system::{VARIABLE_NON_VOLATILE, VARIABLE_BOOTSERVICE_ACCESS, VARIABLE_RUNTIME_ACCESS};

use r_efi::protocols::simple_file_system::Protocol as SimpleFileSystemProtocol;


use core::mem::size_of;

use crate::pi::hob::*;
use crate::pi::fv_lib::*;
use crate::mem::MemoryRegion;

use crate::efi::alloc::Allocator;


use crate::pci;
use crate::part;
use crate::fat;

#[cfg(not(test))]
const VIRTIO_PCI_VENDOR_ID: u16 = 0x1af4;
#[cfg(not(test))]
const VIRTIO_PCI_BLOCK_DEVICE_ID: u16 = 0x1042;

#[cfg(not(test))]
#[repr(C,packed)]
pub struct HardDriveDevicePathNode {
  pub header : DevicePathProtocol,
  pub partition_number: u32,
  pub partition_start: u64,
  pub partition_size: u64,
  pub partition_signature: [u64;2],
  pub partition_format: u8,
  pub partition_type: u8,
}
#[cfg(not(test))]
#[repr(C,packed)]
pub struct HardDriveDevicePath {
  file_system_path_node : HardDriveDevicePathNode,
  end: EndDevicePath,
}

#[cfg(not(test))]
#[repr(C,packed)]
pub struct PciDevicePathNode {
  pub header: DevicePathProtocol,
  pub function: u8,
  pub device: u8,
}


#[cfg(not(test))]
#[repr(C,packed)]
pub struct PciDevicePath {
  pci_device_path_node : PciDevicePathNode,
  end: EndDevicePath,
}

use r_efi::{eficall, eficall_abi};

use core::ffi::c_void;

use crate::pi::hob::{
  Header, MemoryAllocation, ResourceDescription,
  RESOURCE_SYSTEM_MEMORY, HOB_TYPE_MEMORY_ALLOCATION, HOB_TYPE_RESOURCE_DESCRIPTOR, HOB_TYPE_END_OF_HOB_LIST
  };

use handle_database::HandleDatabase;
use variable::Variable;
use variable::MAX_VARIABLE_NAME;
use variable::MAX_VARIABLE_DATA;
use image::Image;
use event::EventInfo;
use conout::ConOut;
use conin::ConIn;

#[cfg(not(test))]
#[repr(C,packed)]
pub struct FullMemoryMappedDevicePath {
  memory_map : MemoryMappedDevicePathProtocol,
  end: EndDevicePath,
}

#[cfg(not(test))]
#[derive(Copy, Clone, PartialEq)]
enum HandleType {
    None,
    Block,
    FileSystem,
    LoadedImage,
}

#[cfg(not(test))]
#[repr(C)]
#[derive(Copy, Clone)]
struct HandleWrapper {
    handle_type: HandleType,
}

lazy_static! {
    pub static ref ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());
}

lazy_static! {
    pub static ref HANDLE_DATABASE: Mutex<HandleDatabase> = Mutex::new(HandleDatabase::new());
}

lazy_static! {
    pub static ref VARIABLE: Mutex<Variable> = Mutex::new(Variable::new());
}

lazy_static! {
    pub static ref IMAGE: Mutex<Image> = Mutex::new(Image::new());
}

lazy_static! {
    pub static ref EVENT: Mutex<EventInfo> = Mutex::new(EventInfo::new());
}

lazy_static! {
    pub static ref CONOUT: Mutex<ConOut> = Mutex::new(ConOut::new());
}

lazy_static! {
    pub static ref CONIN: Mutex<ConIn> = Mutex::new(ConIn::new());
}

#[cfg(not(test))]
pub static mut BLOCK_WRAPPERS: block::BlockWrappers = block::BlockWrappers {
    wrappers: [core::ptr::null_mut(); 16],
    count: 0,
};

#[cfg(not(test))]
pub const BLOCK_PROTOCOL_GUID: Guid = Guid::from_fields(
    0x964e_5b21,
    0x6459,
    0x11d2,
    0x8e,
    0x39,
    &[0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

pub fn print_guid (
    guid: *mut Guid,
    )
{
    let guid_data = unsafe { (*guid).as_fields() };
    crate::log!(
      "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
      guid_data.0,
      guid_data.1,
      guid_data.2,
      guid_data.3,
      guid_data.4,
      guid_data.5[0],
      guid_data.5[1],
      guid_data.5[2],
      guid_data.5[3],
      guid_data.5[4],
      guid_data.5[5]
      );
}

pub fn get_char16_size (
    message: *mut Char16,
    max_size: usize
    ) -> usize
{
    let mut i: usize = 0;
    loop {
        if (i >= max_size) {
            break;
        }
        let output = (unsafe { *message.add(i) } & 0xffu16) as u8;
        i += 1;
        if output == 0 {
            break;
        }
    }
    return i
}

pub fn char16_to_char8 (
    in_message: *mut Char16,
    in_message_size: usize,
    out_message: *mut u8,
    out_message_size: usize,
    ) -> usize
{
    let mut i: usize = 0;
    loop {
        if (i >= in_message_size) {
            break;
        }
        if (i >= out_message_size) {
            break;
        }
        let output = (unsafe { *in_message.add(i) } & 0xffu16) as u8;
        unsafe { *out_message.add(i) = output; }
        i += 1;
        if output == 0 {
            break;
        }
    }
    return i;
}

pub fn print_char16 (
    message: *mut Char16,
    max_size: usize
    ) -> usize
{
    let mut i: usize = 0;
    loop {
        if (i >= max_size) {
            break;
        }
        let output = (unsafe { *message.add(i) } & 0xffu16) as u8;
        i += 1;
        if output == 0 {
            break;
        } else {
            crate::log!("{}", output as char);
        }
    }
    return i;
}

#[cfg(not(test))]
pub extern "win64" fn stdin_reset(_: *mut SimpleTextInputProtocol, _: Boolean) -> Status {
    crate::log!("EFI_STUB: stdin_reset\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdin_read_key_stroke(
    _: *mut SimpleTextInputProtocol,
    key: *mut InputKey,
) -> Status {
    let byte = CONIN.lock().read_byte();
    if byte == 0 {
      unsafe {
        (*key).scan_code = 0;
        (*key).unicode_char = 0;
      }
      return Status::NOT_READY;
    }

    if false {
        let mut string : [Char16; 8] = ['r' as Char16, 'e' as Char16, 'a' as Char16, 'd' as Char16, 0, 0, '\r' as Char16, 0];
        let c = (byte >> 4) & 0xF;
        if (c >= 0xa) {
          string[4] = (c - 0xau8 + 'a' as u8) as u16;
        } else {
          string[4] = (c + '0' as u8) as u16;
        }
        let c = byte & 0xF;
        if (c >= 0xa) {
          string[5] = (c - 0xau8 + 'a' as u8) as u16;
        } else {
          string[5] = (c + '0' as u8) as u16;
        }
        stdout_output_string (unsafe {&mut STDOUT}, &mut string as *mut [Char16; 8] as *mut u16);
    }

    unsafe {
      (*key).scan_code = 0;
      (*key).unicode_char = byte as Char16;
    }

    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdin_reset_ex(_: *mut SimpleTextInputExProtocol, _: Boolean) -> Status {
    crate::log!("EFI_STUB: stdin_reset_ex\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdin_read_key_stroke_ex(
    _: *mut SimpleTextInputExProtocol,
    key_data: *mut KeyData,
) -> Status {
    let byte = CONIN.lock().read_byte();
    if byte == 0 {
      unsafe {
        (*key_data).key.scan_code = 0;
        (*key_data).key.unicode_char = 0;
        (*key_data).key_state.key_shift_state = 0;
        (*key_data).key_state.key_toggle_state = 0;
      }
      return Status::NOT_READY;
    }

    unsafe {
      (*key_data).key.scan_code = 0;
      (*key_data).key.unicode_char = byte as Char16;
      (*key_data).key_state.key_shift_state = 0;
      (*key_data).key_state.key_toggle_state = 0;
    }

    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdin_set_state(
    _: *mut SimpleTextInputExProtocol,
    _: *mut KeyToggleState,
) -> Status {
    crate::log!("EFI_STUB: stdin_set_state\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn stdin_register_key_notify(
    _: *mut SimpleTextInputExProtocol,
    _: *mut KeyData,
    _: KeyNotifyFunction,
    _: *mut *mut core::ffi::c_void,
) -> Status {
    crate::log!("EFI_STUB: stdin_register_key_notify\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdin_unregister_key_notify(
    _: *mut SimpleTextInputExProtocol,
    _: *mut core::ffi::c_void,
) -> Status {
    crate::log!("EFI_STUB: stdin_unregister_key_notify\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn stdout_reset(_: *mut SimpleTextOutputProtocol, _: Boolean) -> Status {
    crate::log!("EFI_STUB: stdout_reset\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_output_string(
    _: *mut SimpleTextOutputProtocol,
    message: *mut Char16,
) -> Status {

    CONOUT.lock().output_string(message);

    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_test_string(
    _: *mut SimpleTextOutputProtocol,
    message: *mut Char16,
) -> Status {
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_query_mode(
    _: *mut SimpleTextOutputProtocol,
    mode_number: usize,
    columns: *mut usize,
    raws: *mut usize,
) -> Status {
    if columns == core::ptr::null_mut() || raws == core::ptr::null_mut() {
      return Status::INVALID_PARAMETER;
    }
    match mode_number {
      0 => {
        unsafe {
        *columns = 80;
        *raws = 25;
        }
      },
      1 => {
        unsafe {
        *columns = 80;
        *raws = 50;
        }
      },
      _ => { return Status::UNSUPPORTED; },
    }
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_set_mode(_: *mut SimpleTextOutputProtocol, mode_number: usize) -> Status {
    CONOUT.lock().set_mode(mode_number)
}

#[cfg(not(test))]
pub extern "win64" fn stdout_set_attribute(_: *mut SimpleTextOutputProtocol, attribute: usize) -> Status {
    CONOUT.lock().set_attribute(attribute);
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_clear_screen(_: *mut SimpleTextOutputProtocol) -> Status {
    CONOUT.lock().clear_screen();
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_set_cursor_position(
    _: *mut SimpleTextOutputProtocol,
    column: usize,
    row: usize,
) -> Status {
    CONOUT.lock().set_cursor_position(column, row);
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn stdout_enable_cursor(_: *mut SimpleTextOutputProtocol, visible: Boolean) -> Status {
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn get_time(_: *mut Time, _: *mut TimeCapabilities) -> Status {
    crate::log!("EFI_STUB: get_time - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn set_time(_: *mut Time) -> Status {
    crate::log!("EFI_STUB: set_time - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn get_wakeup_time(_: *mut Boolean, _: *mut Boolean, _: *mut Time) -> Status {
    crate::log!("EFI_STUB: get_wakeup_time - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn set_wakeup_time(_: Boolean, _: *mut Time) -> Status {
    crate::log!("EFI_STUB: set_wakeup_time - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn set_virtual_address_map(
    map_size: usize,
    descriptor_size: usize,
    version: u32,
    descriptors: *mut MemoryDescriptor,
) -> Status {
    crate::log!("EFI_STUB: set_virtual_address_map - ???\n");
    let count = map_size / descriptor_size;

    if version != efi::MEMORY_DESCRIPTOR_VERSION {
        return Status::INVALID_PARAMETER;
    }

    let descriptors = unsafe {
        core::slice::from_raw_parts_mut(descriptors as *mut alloc::MemoryDescriptor, count)
    };

    ALLOCATOR.lock().update_virtual_addresses(descriptors)
}

#[cfg(not(test))]
pub extern "win64" fn convert_pointer(_: usize, _: *mut *mut c_void) -> Status {
    crate::log!("EFI_STUB: convert_pointer - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn get_variable(
    var_name: *mut Char16,
    var_guid: *mut Guid,
    attributes: *mut u32,
    size: *mut usize,
    data: *mut core::ffi::c_void,
) -> Status {
    let var_name_size = get_char16_size (var_name, core::usize::MAX);
    if false {
      crate::log!("EFI_STUB: get_variable ");
      print_char16 (var_name, var_name_size);
      crate::log!(" ");
      print_guid (var_guid);
      crate::log!("\n");
    }

    if var_name_size > MAX_VARIABLE_NAME {
      crate::log!("name too long\n");
      return Status::UNSUPPORTED;
    }

    let mut name_buffer: [u8; MAX_VARIABLE_NAME] = [0; MAX_VARIABLE_NAME];
    char16_to_char8 (
        var_name,
        var_name_size,
        &mut name_buffer as *mut [u8; MAX_VARIABLE_NAME] as *mut u8,
        MAX_VARIABLE_NAME);

    let guid_buffer : *mut [u8; 16] = unsafe { core::mem::transmute::<*mut Guid, *mut [u8; 16]>(var_guid) };

    let (status, var_attributes, var_size, var_data) = VARIABLE.lock().get_variable(
                     &mut name_buffer as *mut [u8; MAX_VARIABLE_NAME],
                     guid_buffer
                     );

    if (status == Status::NOT_FOUND) {
      return status;
    }

    if unsafe {*size} < var_size {
      unsafe {*size = var_size;}
      return Status::BUFFER_TOO_SMALL;
    }

    unsafe {*size = var_size;}
    let data_ptr : *mut c_void = unsafe { core::mem::transmute::<*mut [u8; MAX_VARIABLE_DATA], *mut c_void>(var_data) };
    unsafe {core::ptr::copy_nonoverlapping (data_ptr, data, var_size);}

    if attributes != core::ptr::null_mut() {
      unsafe {*attributes = var_attributes;}
    }

    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn get_next_variable_name(
    _: *mut usize,
    _: *mut Char16,
    _: *mut Guid,
) -> Status {
    crate::log!("EFI_STUB: get_next_variable - UNSUPPORTED, TODO\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn set_variable(
    var_name: *mut Char16,
    var_guid: *mut Guid,
    attributes: u32,
    size: usize,
    data: *mut c_void,
) -> Status {
    let var_name_size = get_char16_size (var_name, core::usize::MAX);
    if false {
      crate::log!("EFI_STUB: set_variable ");
      print_char16 (var_name, var_name_size);
      crate::log!(" ");
      print_guid (var_guid);
      crate::log!("\n");
    }

    if var_name_size > MAX_VARIABLE_NAME {
      crate::log!("name too long\n");
      return Status::UNSUPPORTED;
    }

    let mut name_buffer: [u8; MAX_VARIABLE_NAME] = [0; MAX_VARIABLE_NAME];
    char16_to_char8 (
        var_name,
        var_name_size,
        &mut name_buffer as *mut [u8; MAX_VARIABLE_NAME] as *mut u8,
        MAX_VARIABLE_NAME);

    let guid_buffer : *mut [u8; 16] = unsafe { core::mem::transmute::<*mut Guid, *mut [u8; 16]>(var_guid) };

    if size > MAX_VARIABLE_DATA {
      crate::log!("data too long\n");
      return Status::UNSUPPORTED;
    }

    let data_buffer: *mut [u8; MAX_VARIABLE_DATA] = unsafe { core::mem::transmute::<*mut c_void, *mut [u8; MAX_VARIABLE_DATA]>(data) };

    let (status) = VARIABLE.lock().set_variable(
                     &mut name_buffer as *mut [u8; MAX_VARIABLE_NAME],
                     guid_buffer,
                     attributes,
                     size,
                     data_buffer
                     );

    status
}

#[cfg(not(test))]
pub extern "win64" fn get_next_high_mono_count(_: *mut u32) -> Status {
    crate::log!("EFI_STUB: get_next_high_mono_count - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn reset_system(_: ResetType, _: Status, _: usize, _: *mut c_void) {
    crate::log!("EFI_STUB: reset_system.\n");
    crate::i8042_reset();
}

#[cfg(not(test))]
pub extern "win64" fn update_capsule(
    _: *mut *mut CapsuleHeader,
    _: usize,
    _: PhysicalAddress,
) -> Status {
    crate::log!("EFI_STUB: update_capsule - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn query_capsule_capabilities(
    _: *mut *mut CapsuleHeader,
    _: usize,
    _: *mut u64,
    _: *mut ResetType,
) -> Status {
    crate::log!("EFI_STUB: query_capsule_capabilities - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn query_variable_info(_: u32, _: *mut u64, _: *mut u64, _: *mut u64) -> Status {
    crate::log!("EFI_STUB: query_variable_info - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn raise_tpl(_: Tpl) -> Tpl {
    crate::log!("EFI_STUB: raise_tpl\n");
    0
}

#[cfg(not(test))]
pub extern "win64" fn restore_tpl(_: Tpl) {
    crate::log!("EFI_STUB: restore_tpl\n");
}

#[cfg(not(test))]
pub extern "win64" fn allocate_pages(
    allocate_type: AllocateType,
    memory_type: MemoryType,
    pages: usize,
    address: *mut PhysicalAddress,
) -> Status {
    let (status, new_address) =
        ALLOCATOR
            .lock()
            .allocate_pages(
                allocate_type,
                memory_type,
                pages as u64,
                unsafe { *address } as u64,
            );
    if status == Status::SUCCESS {
        unsafe {
            *address = new_address;
        }
    } else {
      log!("allocate pages status - {:?}\n", status);
    }
    status
}

#[cfg(not(test))]
pub extern "win64" fn free_pages(address: PhysicalAddress, _: usize) -> Status {
    ALLOCATOR.lock().free_pages(address)
}

#[cfg(not(test))]
pub extern "win64" fn get_memory_map(
    memory_map_size: *mut usize,
    out: *mut MemoryDescriptor,
    key: *mut usize,
    descriptor_size: *mut usize,
    descriptor_version: *mut u32,
) -> Status {
    log!("EFI_STUB - get_memory_map\n");
    let count = ALLOCATOR.lock().get_descriptor_count();
    let map_size = core::mem::size_of::<MemoryDescriptor>() * count;
    if unsafe { *memory_map_size } < map_size {
        unsafe {
            *memory_map_size = map_size;
        }
        return Status::BUFFER_TOO_SMALL;
    }

    let out =
        unsafe { core::slice::from_raw_parts_mut(out as *mut alloc::MemoryDescriptor, count) };
    let count = ALLOCATOR.lock().get_descriptors(out);
    let map_size = core::mem::size_of::<MemoryDescriptor>() * count;
    unsafe {
        *memory_map_size = map_size;
        *descriptor_version = efi::MEMORY_DESCRIPTOR_VERSION;
        *descriptor_size = core::mem::size_of::<MemoryDescriptor>();
        *key = ALLOCATOR.lock().get_map_key();
    }

    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn allocate_pool(
    memory_type: MemoryType,
    size: usize,
    address: *mut *mut c_void,
) -> Status {
    let (status, new_address) = ALLOCATOR.lock().allocate_pages(
        AllocateType::AllocateAnyPages,
        memory_type,
        ((size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize) as u64,
        address as u64,
    );

    if status == Status::SUCCESS {
        unsafe {
            *address = new_address as *mut c_void;
        }
    } else {
      log!("allocate pool status - {:?}\n", status);
    }

    status
}

#[cfg(not(test))]
pub extern "win64" fn free_pool(ptr: *mut c_void) -> Status {
    ALLOCATOR.lock().free_pages(ptr as u64)
}

#[cfg(not(test))]
pub extern "win64" fn create_event(
    r#type: u32,
    notify_tpl: Tpl,
    notify_function: EventNotify,
    notify_context: *mut c_void,
    event: *mut Event,
) -> Status {

    let (status, new_event) = EVENT.lock().create_event(
            r#type,
            notify_tpl,
            notify_function,
            notify_context
            );
    crate::log!("EFI_STUB: create_event - type:0x{:x} tpl:0x{:x} - status: {:?}\n", r#type, notify_tpl as usize, status);
    if status == Status::SUCCESS {
        unsafe {
            *event = new_event;
        }
    }
    status
}

#[cfg(not(test))]
pub extern "win64" fn set_timer(_: Event, _: TimerDelay, _: u64) -> Status {
    crate::log!("EFI_STUB: set_timer - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn wait_for_event(_: usize, _: *mut Event, _: *mut usize) -> Status {
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn signal_event(_: Event) -> Status {
    crate::log!("EFI_STUB: signal_event - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn close_event(_: Event) -> Status {
    crate::log!("EFI_STUB: close_event - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn check_event(_: Event) -> Status {
    crate::log!("EFI_STUB: check_event - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn install_protocol_interface(
    handle: *mut Handle,
    guid: *mut Guid,
    interface_type: InterfaceType,
    interface: *mut c_void,
) -> Status {

    let (status, new_handle) = HANDLE_DATABASE.lock().install_protocol(
                unsafe {*handle},
                guid,
                interface,
            );
    crate::log!("EFI_STUB: install_protocol_interface: {:?}, handle: {:?}, interface: {:?} - new_handle: {:?} status: {:?}\n", unsafe{*guid}, unsafe{*handle}, interface, new_handle, status);
    if status == Status::SUCCESS {
        unsafe {
            *handle = new_handle;
        }
    }
    status
}

#[cfg(not(test))]
pub extern "win64" fn reinstall_protocol_interface(
    _: Handle,
    _: *mut Guid,
    _: *mut c_void,
    _: *mut c_void,
) -> Status {
    crate::log!("EFI_STUB: reinstall_protocol_interface - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn uninstall_protocol_interface(
    _: Handle,
    _: *mut Guid,
    _: *mut c_void,
) -> Status {
    crate::log!("EFI_STUB: uninstall_protocol_interface - UNSUPPORTED, TODO\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn handle_protocol(
    handle: Handle,
    guid: *mut Guid,
    out: *mut *mut c_void,
) -> Status {
    if guid == core::ptr::null_mut() {
        crate::log!("EFI_STUB: handle_protocol - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    let (status, interface) = HANDLE_DATABASE.lock().handle_protocol(handle, guid);

    if ! (unsafe{*guid} == r_efi::protocols::simple_text_input_ex::PROTOCOL_GUID ||
        unsafe{*guid} == r_efi::protocols::simple_text_output::PROTOCOL_GUID)
    {
        crate::log!("EFI_STUB - handle_protocol: {:?}, handle: {:?} - status {:x}, interface: {:?}\n", unsafe{*guid}, handle, status.value(), interface);
    }
    if status == Status::SUCCESS {
        unsafe {
            *out = interface;
        }
    }
    status
}

#[cfg(not(test))]
pub extern "win64" fn register_protocol_notify(
    _: *mut Guid,
    _: Event,
    _: *mut *mut c_void,
) -> Status {
    crate::log!("EFI_STUB: register_protocol_notify - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn locate_handle(
    locate_search_type: LocateSearchType,
    guid: *mut Guid,
    search_key: *mut c_void,
    buffer_size: *mut usize,
    buffer: *mut Handle,
) -> Status {
    if guid == core::ptr::null_mut() {
        crate::log!("EFI_STUB: locate_handle - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    if locate_search_type as u32 != LocateSearchType::ByProtocol as u32 {
      log!("locate_search_type - {}\n", locate_search_type as u32);
      return Status::UNSUPPORTED;
    }
    if search_key != core::ptr::null_mut() {
      log!("search_key - {:p}\n", search_key);
      return Status::UNSUPPORTED;
    }

    let input_buffer_size = unsafe { *buffer_size };
    let (status, final_buffer_size) = HANDLE_DATABASE.lock().locate_handle(guid, input_buffer_size, buffer);
    crate::log!("EFI_STUB: locate_handle - guid: {:?}, buffer_size: {:?} - status: {:x}, buffer_size: {:?}\n", unsafe{*guid}, unsafe{*buffer_size}, status.value(), final_buffer_size);
    match status {
      Status::SUCCESS => {},
      Status::BUFFER_TOO_SMALL => {},
      Status::NOT_FOUND => {},
      _ => {crate::log!("EFI_STUB: locate_handle error\n");return status;}
    }

    unsafe { *buffer_size = final_buffer_size; }

    status
}

#[cfg(not(test))]
pub extern "win64" fn locate_device_path(protocol: *mut Guid, device_path: *mut *mut c_void, device: *mut Handle) -> Status {

    let source_path: *mut DevicePathProtocol = unsafe{*device_path as *mut DevicePathProtocol};
    // crate::log!("EFI_STUB: locate_device_path protocol: {:?}, devicePath address: {:?} value {:?}\n", unsafe{*protocol}, source_path, unsafe{*source_path});

    if device_path == core::ptr::null_mut() {
        crate::log!("EFI_STUB: locate_device_path: device_path is NULL\n");
        return   Status::INVALID_PARAMETER;
    }

    if unsafe{*device_path} == core::ptr::null_mut() {
        crate::log!("EFI_STUB: locate_device_path: *device_path is NULL\n");
        return Status::INVALID_PARAMETER;
    }


    let mut tmp_device_path: *mut DevicePathProtocol = unsafe{*device_path as *mut DevicePathProtocol};
    let mut source_path = unsafe{*device_path as *mut DevicePathProtocol};

    while !crate::efi::device_path::is_device_path_end(tmp_device_path) {
        if crate::efi::device_path::is_device_path_end_instance(tmp_device_path) {
            break;
        }
        tmp_device_path = crate::efi::device_path::get_next_device_path_node(tmp_device_path);
    }

    let source_size = tmp_device_path as *mut c_void as u64 - source_path as *mut c_void as u64;
    // crate::log!("EFI_STUB: locate_device_path: source_size is {}\n", source_size);

    let (status, handle_count, handle_buffer) = HANDLE_DATABASE.lock().locate_handle_buffer(protocol);
    if status != Status::SUCCESS || handle_count == 0 {
        crate::log!("EFI_STUB: locate_device_path: not found\n");
        return Status::NOT_FOUND;
    }
    let handles = handle_buffer as *mut [Handle; 128];
    let mut handle: Handle;

    let mut best_match: i32 = -1;
    let mut best_device: Handle = core::ptr::null_mut();

    for index in 0 .. handle_count {
        unsafe{handle = (*handles)[index];}
        let (status, interface) = HANDLE_DATABASE.lock().handle_protocol(handle,
            &mut r_efi::protocols::device_path::PROTOCOL_GUID as *mut Guid);
        if status != Status::SUCCESS {
            crate::log!("EFI_STUB: locate_device_path: error {:?}\n", status);
            continue;
        }

        // crate::log!("EFI_STUB: locate_device_path: interface address: {:?}, interface data: {:?}\n", interface, unsafe{*(interface as *mut DevicePathProtocol)});
        let size = crate::efi::device_path::get_device_path_size(interface as *mut DevicePathProtocol) - 4;

        if (size as u64 <= source_size) && crate::efi::device_path::compare_device_path(interface as *mut DevicePathProtocol, source_path, size) == true {
            if size as i32 == best_match {
                crate::log!("EFI_STUB: locate_device_path: duplicate device path for 2 different device handles\n");
            }

            if size as i32 > best_match {
                best_match = size as i32;
                best_device = handle;
            }
        }
    }

    if best_match == -1 {
        crate::log!("EFI_STUB: locate_device_path: not found\n");
        return Status::NOT_FOUND;
    }

    if device == core::ptr::null_mut() {
        return Status::INVALID_PARAMETER;
    }

    unsafe {
        *device = best_device;
        let dp = (source_path as u64 + best_match as u64) as *mut DevicePathProtocol;
        *device_path = dp as *mut c_void;
        // crate::log!("EFI_STUB: locate_device_path: device_path address {:?}, device_path {:?}, device: {:?}\n", dp, *dp, best_device);
    }

    Status::SUCCESS

}

#[cfg(not(test))]
pub extern "win64" fn install_configuration_table(_: *mut Guid, _: *mut c_void) -> Status {
    crate::log!("EFI_STUB: install_configuration_table - UNSUPPORTED\n");

    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn load_image(
    boot_policy: Boolean,
    parent_image_handle: Handle,
    device_path: *mut c_void,
    source_buffer: *mut c_void,
    source_size: usize,
    image_handle: *mut Handle,
) -> Status {
    crate::log!("EFI_STUB: load_image size is: {}, parent_image_handle: {:?}\n", source_size, parent_image_handle);
    let mut source_buffer = source_buffer;
    let mut source_size = source_size;

    if source_size == 0 {
        //device_path::print_device_path(device_path as *mut efi::protocols::device_path::Protocol);
        if let Some(filename) = crate::efi::device_path::get_file_path_media_device_path(device_path as *mut efi::protocols::device_path::Protocol) {
            let mut name = [0u8;512];
            char16_to_char8(filename, 256, &name[0] as *const u8 as *mut u8, 256);
            log!("EFI_STUB: filename is {}\n", core::str::from_utf8(&name[..]).unwrap_or("error"));
            let mut fs_interface = core::ptr::null_mut();
            //let status = handle_protocol(parent_image_handle, &efi::protocols::simple_file_system::PROTOCOL_GUID as *const efi::Guid as *mut efi::Guid, &mut fs_interface);
            let mut handle = core::ptr::null_mut();
            crate::log!("EFI_STUB: start locate_protocol\n");
            let status = locate_protocol(&efi::protocols::simple_file_system::PROTOCOL_GUID as *const efi::Guid as *mut efi::Guid, handle, &mut fs_interface);
            crate::log!("EFI_STUB: simple_file_system protocol 0x{:p} status: {:x}\n", fs_interface, status.value());
            let mut fs = fs_interface as *mut efi::protocols::simple_file_system::Protocol;
            let mut rootfile = core::ptr::null_mut() as *mut efi::protocols::file::Protocol;
            let mut desfile = core::ptr::null_mut() as *mut efi::protocols::file::Protocol;
            let mut status: efi::Status = unsafe{((*fs).open_volume)(fs, &mut rootfile)};
            if status.is_error() {
                log!("EFI_STUB: load image open_volume error \n");
            }

            unsafe{
                status = ((*rootfile).open)(rootfile, &mut desfile as *mut *mut efi::protocols::file::Protocol, filename as *mut efi::Char16, efi::protocols::file::MODE_READ, 0);
                if status.is_error() {
                    log!("EFI_STUB: load image open error \n");
                }
                let mut buffer = [0u8; core::mem::size_of::<efi::protocols::file::Info>() + 1024];
                let mut buffer_size:usize = core::mem::size_of::<efi::protocols::file::Info>() + 1024;
                status = ((*desfile).get_info)(desfile, &mut efi::protocols::file::INFO_ID as *mut efi::Guid, &mut buffer_size, &mut buffer[0] as *mut u8 as *mut core::ffi::c_void);
                if status.is_error() {
                    log!("EFI_STUB: load image get_info error 0x{:x}\n", status.value());
                }
                let desfile_info = &buffer as *const u8 as *mut efi::protocols::file::Info;
                source_size = unsafe{(*desfile_info).file_size} as usize;
                log!("EFI_STUB: file size is: {:?}\n", source_size);

                source_buffer = core::ptr::null_mut();
                status = crate::efi::allocate_pool (MemoryType::BootServicesData, source_size, &mut source_buffer);
                if status != Status::SUCCESS {
                  log!("load_image - fail on allocate pool\n");
                  return Status::OUT_OF_RESOURCES;
                }

                status = ((*desfile).read)(desfile, &mut source_size, source_buffer);
                if status.is_error() {
                    log!("EFI_STUB: load image read error 0x{:x}\n", status.value());
                }
            }

        } else {
            log!("EFI_STUB: not found filename\n");
        }
        // simple file system device path

    }

    //let (status, new_image_handle) = IMAGE.lock().load_image(
    let (status, new_image_handle) = Image::new().load_image(
        parent_image_handle,
        device_path,
        source_buffer,
        source_size,
    );

    crate::log!("EFI_STUB: load_image done handle {:?} status 0x{:x}\n", new_image_handle, status.value());
    if status == Status::SUCCESS {
        if image_handle != core::ptr::null_mut() {
          unsafe { *image_handle = new_image_handle };
        };
    }

    status
}

#[cfg(not(test))]
pub extern "win64" fn start_image(
    image_handle: Handle,
    exit_data_size: *mut usize,
    exit_data: *mut *mut Char16
) -> Status {
    crate::log!("EFI_STUB: start_image, handle: {:?}\n", image_handle);

    let (status, new_exit_data_size, new_exit_data) = Image::new().start_image(image_handle);

    if exit_data_size != core::ptr::null_mut() {
      unsafe { *exit_data_size = new_exit_data_size };
    }
    if exit_data != core::ptr::null_mut() {
      unsafe { *exit_data = new_exit_data };
    }

    status
}

#[cfg(not(test))]
pub extern "win64" fn exit(_: Handle, _: Status, _: usize, _: *mut Char16) -> Status {
    crate::log!("EFI_STUB: exit - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn unload_image(_: Handle) -> Status {
    crate::log!("EFI_STUB: unload_image - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn exit_boot_services(_: Handle, _: usize) -> Status {
    crate::log!("EFI_STUB: exit_boot_services\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn get_next_monotonic_count(_: *mut u64) -> Status {
    crate::log!("EFI_STUB: get_next_monotonic_count - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn stall(_: usize) -> Status {
    crate::log!("EFI_STUB: stall - called\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn set_watchdog_timer(_: usize, _: u64, _: usize, _: *mut Char16) -> Status {
    crate::log!("EFI_STUB: set_watchdog_timer\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn connect_controller(
    _: Handle,
    _: *mut Handle,
    _: *mut c_void,
    _: Boolean,
) -> Status {
    crate::log!("EFI_STUB: connect_controller - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn disconnect_controller(_: Handle, _: Handle, _: Handle) -> Status {
    crate::log!("EFI_STUB: disconnect_controller - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn open_protocol(
    handle: Handle,
    guid: *mut Guid,
    out_interface: *mut *mut c_void,
    agent_handle: Handle,
    controller_handle: Handle,
    attributes: u32,
) -> Status {
    if guid == core::ptr::null_mut() {
        crate::log!("EFI_STUB: open_protocol - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    if attributes != OPEN_PROTOCOL_GET_PROTOCOL {
        crate::log!("EFI_STUB - open_protocol: attribute not support\n");
        return Status::UNSUPPORTED;
    }

    unsafe {*out_interface = core::ptr::null_mut();}
    let (status, interface) = HANDLE_DATABASE.lock().handle_protocol (handle, guid);
    crate::log!("EFI_STUB - open_protocol: {:?}, handle: {:?}, attributes: {} - return - status: {:?}, interface {:?}\n", unsafe{*guid}, handle, attributes, status, interface);
    if status == Status::SUCCESS {
      unsafe {*out_interface = interface;}
    }

    status
}

#[cfg(not(test))]
pub extern "win64" fn close_protocol(_: Handle, _: *mut Guid, _: Handle, _: Handle) -> Status {
    crate::log!("EFI_STUB: close_protocol\n");
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn open_protocol_information(
    _: Handle,
    _: *mut Guid,
    _: *mut *mut OpenProtocolInformationEntry,
    _: *mut usize,
) -> Status {
    crate::log!("EFI_STUB: open_protocol_information - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn protocols_per_handle(
    _: Handle,
    _: *mut *mut *mut Guid,
    _: *mut usize,
) -> Status {
    crate::log!("EFI_STUB: protocols_per_handle - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn locate_handle_buffer(
    locate_search_type: LocateSearchType,
    guid: *mut Guid,
    search_key: *mut c_void,
    no_handles: *mut usize,
    buffer: *mut *mut Handle,
) -> Status {
    if guid == core::ptr::null_mut() {
        crate::log!("EFI_STUB: locate_handle_buffer - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    // crate::log!("EFI_STUB: locate_handle_buffer - ");
    // print_guid (guid);
    // crate::log!("\n");

    if locate_search_type as u32 != LocateSearchType::ByProtocol as u32 {
      log!("locate_search_type - {}\n", locate_search_type as u32);
      return Status::UNSUPPORTED;
    }
    if search_key != core::ptr::null_mut() {
      log!("search_key - {:p}\n", search_key);
      return Status::UNSUPPORTED;
    }

    let (status, handle_count, handle_buffer) = HANDLE_DATABASE.lock().locate_handle_buffer(guid);
    if status == Status::SUCCESS {
        unsafe {
            *no_handles = handle_count;
            *buffer = handle_buffer as *mut Handle;
        }
    }
    log!("status - {:?}\n", status);
    status
}

#[cfg(not(test))]
pub extern "win64" fn locate_protocol(guid: *mut Guid, registration: *mut c_void, interface: *mut *mut c_void) -> Status {
    crate::log!("EFI_STUB: locate_protocol - {:?}\n", unsafe{*guid});
    if guid == core::ptr::null_mut() {
        crate::log!("EFI_STUB: locate_protocol - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    let (status, new_interface) = HANDLE_DATABASE.lock().locate_protocol(guid);
    if status == Status::SUCCESS {
      unsafe {*interface = new_interface; }
    } else {
      crate::log!("EFI_STUB - locate_protocol: {:?} failed, status: {:?}\n", unsafe{*guid}, status);
    }

    status
}
//
// NOTE:
// see https://github.com/rust-lang/rfcs/blob/master/text/2137-variadic.md
// Current vararg support only "C".
// "win64" is not supported.
//
// As such we cannot use below:
// pub unsafe extern "C" fn install_multiple_protocol_interfaces(
//    handle: *mut Handle,
//    mut args: ...
// ) -> Status;
//
// NOTE: Current EDKII has use case with 5 guid/interface pairs.
// So we hardcode to support 8 pairs as maximum. It should be enought.
//
#[cfg(not(test))]
pub extern "win64" fn install_multiple_protocol_interfaces_real(
    handle: *mut Handle,
    guid1: *mut Guid,
    interface1: *mut c_void,
    guid2: *mut Guid,
    interface2: *mut c_void,
    guid3: *mut Guid,
    interface3: *mut c_void,
    guid4: *mut Guid,
    interface4: *mut c_void,
    guid5: *mut Guid,
    interface5: *mut c_void,
    guid6: *mut Guid,
    interface6: *mut c_void,
    guid7: *mut Guid,
    interface7: *mut c_void,
    guid8: *mut Guid,
    interface8: *mut c_void,
    guid_null: *mut c_void,
) -> Status {
    let mut count : usize = 0;
    let mut pair : [(*mut Guid, *mut c_void); 8] = [(core::ptr::null_mut(), core::ptr::null_mut()) ; 8];

    if guid1 == core::ptr::null_mut() {
      crate::log!("EFI_STUB: install_multiple_protocol_interfaces_real - no GUID/Interface pair\n");
      return Status::INVALID_PARAMETER;
    } else {
      count = 1;
      pair[0] = (guid1, interface1);
    }
    if guid2 != core::ptr::null_mut() {
      count = 2;
      pair[1] = (guid2, interface2);
    }
    if guid3 != core::ptr::null_mut() {
      count = 3;
      pair[2] = (guid3, interface3);
    }
    if guid4 != core::ptr::null_mut() {
      count = 4;
      pair[3] = (guid4, interface4);
    }
    if guid5 != core::ptr::null_mut() {
      count = 5;
      pair[4] = (guid5, interface5);
    }
    if guid6 != core::ptr::null_mut() {
      count = 6;
      pair[5] = (guid6, interface6);
    }
    if guid7 != core::ptr::null_mut() {
      count = 7;
      pair[6] = (guid7, interface7);
    }
    if guid8 != core::ptr::null_mut() {
      count = 8;
      pair[7] = (guid8, interface8);
    }
    if guid_null != core::ptr::null_mut() {
      crate::log!("EFI_STUB: install_multiple_protocol_interfaces_real - too many GUID/Interface pair\n");
      return Status::UNSUPPORTED;
    }

    crate::log!("EFI_STUB: install_multiple_protocol_interfaces_real:\n");
    for index in 0 .. count {
      crate::log!("  ");
      print_guid (pair[index].0);
      crate::log!("  ");
      crate::log!("{:p}", pair[index].1);
      crate::log!("\n");
    }

    let (status, new_handle) = HANDLE_DATABASE.lock().install_multiple_protocol(
                unsafe {*handle},
                count,
                &mut pair
                );
    log!("status - {:?}\n", status);
    if status == Status::SUCCESS {
        unsafe {
            *handle = new_handle;
        }
    }
    status
}

pub extern "win64" fn uninstall_multiple_protocol_interfaces_real(
    handle: *mut Handle,
    guid1: *mut Guid,
    interface1: *mut c_void,
    guid2: *mut Guid,
    interface2: *mut c_void,
    guid3: *mut Guid,
    interface3: *mut c_void,
    guid4: *mut Guid,
    interface4: *mut c_void,
    guid5: *mut Guid,
    interface5: *mut c_void,
    guid6: *mut Guid,
    interface6: *mut c_void,
    guid7: *mut Guid,
    interface7: *mut c_void,
    guid8: *mut Guid,
    interface8: *mut c_void,
    guid_null: *mut c_void,
) -> Status {
    crate::log!("EFI_STUB: uninstall_multiple_protocol_interfaces_real - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn install_multiple_protocol_interfaces(
    handle: *mut Handle,
    guid: *mut c_void,
    interface: *mut c_void,
) -> Status {

    let guid_ptr = guid as *mut Guid;

    crate::log!("EFI_STUB: install_multiple_protocol_interfaces - UNSUPPORTED - ");
    print_guid (guid_ptr);
    crate::log!("\n");

    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn uninstall_multiple_protocol_interfaces(
    _: *mut Handle,
    _: *mut c_void,
    _: *mut c_void,
) -> Status {
    crate::log!("EFI_STUB: uninstall_multiple_protocol_interfaces - UNSUPPORTED\n");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn calculate_crc32(_: *mut c_void, _: usize, _: *mut u32) -> Status {
    Status::SUCCESS
}

#[cfg(not(test))]
pub extern "win64" fn copy_mem(dest: *mut c_void, source: *mut c_void, size: usize) {
    unsafe {core::ptr::copy (source, dest, size);}
}

#[cfg(not(test))]
pub extern "win64" fn set_mem(buffer: *mut c_void, size: usize, val: u8) {
    unsafe {core::ptr::write_bytes (buffer, val, size);}
}

#[cfg(not(test))]
pub extern "win64" fn create_event_ex(
    _: u32,
    _: Tpl,
    _: EventNotify,
    _: *const c_void,
    _: *const Guid,
    event: *mut Event,
) -> Status {
    crate::log!("EFI_STUB: create_event_ex - UNSUPPORTED\n");

    if event == core::ptr::null_mut() {
        crate::log!("EFI_STUB: create_event_ex - NULL\n");
        return Status::INVALID_PARAMETER;
    }

    unsafe {*event = core::ptr::null_mut();}

    // TBD
    Status::SUCCESS
}

#[cfg(not(test))]
extern "win64" fn image_unload(_: Handle) -> Status {
    crate::log!("EFI_STUB: image_unload - UNSUPPORTED\n");
    efi::Status::UNSUPPORTED
}

#[cfg(not(test))]
pub const PAGE_SIZE: u64 = 4096;

pub static mut STDIN : SimpleTextInputProtocol = SimpleTextInputProtocol {
          reset: stdin_reset,
          read_key_stroke: stdin_read_key_stroke,
          wait_for_key: 0 as Event,
      };

pub static mut STDIN_EX : SimpleTextInputExProtocol = SimpleTextInputExProtocol {
          reset: stdin_reset_ex,
          read_key_stroke_ex: stdin_read_key_stroke_ex,
          wait_for_key_ex: 0 as Event,
          set_state: stdin_set_state,
          register_key_notify: stdin_register_key_notify,
          unregister_key_notify: stdin_unregister_key_notify,

      };

pub static mut STDOUT_MODE : SimpleTextOutputMode = SimpleTextOutputMode {
        max_mode: 1,
        mode: 0,
        attribute: 0,
        cursor_column: 0,
        cursor_row: 0,
        cursor_visible: Boolean::FALSE,
      };

pub static mut STDOUT : SimpleTextOutputProtocol = SimpleTextOutputProtocol {
        reset: stdout_reset,
        output_string: stdout_output_string,
        test_string: stdout_test_string,
        query_mode: stdout_query_mode,
        set_mode: stdout_set_mode,
        set_attribute: stdout_set_attribute,
        clear_screen: stdout_clear_screen,
        set_cursor_position: stdout_set_cursor_position,
        enable_cursor: stdout_enable_cursor,
        mode: core::ptr::null_mut(),
      };

pub static mut RT : efi::RuntimeServices = efi::RuntimeServices {
        hdr: efi::TableHeader {
            signature: efi::RUNTIME_SERVICES_SIGNATURE,
            revision: efi::RUNTIME_SERVICES_REVISION,
            header_size: core::mem::size_of::<efi::RuntimeServices>() as u32,
            crc32: 0, // TODO
            reserved: 0,
        },
        get_time,
        set_time,
        get_wakeup_time,
        set_wakeup_time,
        set_virtual_address_map,
        convert_pointer,
        get_variable,
        get_next_variable_name,
        set_variable,
        get_next_high_mono_count,
        reset_system,
        update_capsule,
        query_capsule_capabilities,
        query_variable_info,
      };

pub type InstallMultipleProtocolInterfacesFunc = extern "win64" fn(*mut Handle, *mut c_void, *mut c_void) -> r_efi::base::Status;
pub type UninstallMultipleProtocolInterfacesFunc = extern "win64" fn(*mut Handle,*mut c_void,*mut c_void,) -> r_efi::base::Status;

pub static mut BS : efi::BootServices = efi::BootServices {
        hdr: efi::TableHeader {
            signature: efi::BOOT_SERVICES_SIGNATURE,
            revision: efi::BOOT_SERVICES_REVISION,
            header_size: core::mem::size_of::<efi::BootServices>() as u32,
            crc32: 0, // TODO
            reserved: 0,
        },
        raise_tpl,
        restore_tpl,
        allocate_pages,
        free_pages,
        get_memory_map,
        allocate_pool,
        free_pool,
        create_event,
        set_timer,
        wait_for_event,
        signal_event,
        close_event,
        check_event,
        install_protocol_interface,
        reinstall_protocol_interface,
        uninstall_protocol_interface,
        handle_protocol,
        register_protocol_notify,
        locate_handle,
        locate_device_path,
        install_configuration_table,
        load_image,
        start_image,
        exit,
        unload_image,
        exit_boot_services,
        get_next_monotonic_count,
        stall,
        set_watchdog_timer,
        connect_controller,
        disconnect_controller,
        open_protocol,
        close_protocol,
        open_protocol_information,
        protocols_per_handle,
        locate_handle_buffer,
        locate_protocol,
        install_multiple_protocol_interfaces,
        uninstall_multiple_protocol_interfaces,
        calculate_crc32,
        copy_mem,
        set_mem,
        create_event_ex,
        reserved: core::ptr::null_mut(),
      };

const MAX_CONFIGURATION_TABLE : usize = 4;

pub static mut CT : [efi::ConfigurationTable; MAX_CONFIGURATION_TABLE] =
        [
          efi::ConfigurationTable {
            vendor_guid: Guid::from_fields(0, 0, 0, 0, 0, &[0; 6]), // TODO
            vendor_table: core::ptr::null_mut(),},
          efi::ConfigurationTable {
            vendor_guid: Guid::from_fields(0, 0, 0, 0, 0, &[0; 6]), // TODO
            vendor_table: core::ptr::null_mut(),},
          efi::ConfigurationTable {
            vendor_guid: Guid::from_fields(0, 0, 0, 0, 0, &[0; 6]), // TODO
            vendor_table: core::ptr::null_mut(),},
          efi::ConfigurationTable {
            vendor_guid: Guid::from_fields(0, 0, 0, 0, 0, &[0; 6]), // TODO
            vendor_table: core::ptr::null_mut(),},
        ];

pub static mut ST : efi::SystemTable = efi::SystemTable {
        hdr: efi::TableHeader {
            signature: efi::SYSTEM_TABLE_SIGNATURE,
            revision: efi::SYSTEM_TABLE_REVISION_2_70,
            header_size: core::mem::size_of::<efi::SystemTable>() as u32,
            crc32: 0, // TODO
            reserved: 0,
        },
        firmware_vendor: core::ptr::null_mut(), // TODO,
        firmware_revision: 0,
        console_in_handle: core::ptr::null_mut(),
        con_in: core::ptr::null_mut(),
        console_out_handle: core::ptr::null_mut(),
        con_out: core::ptr::null_mut(),
        standard_error_handle: core::ptr::null_mut(),
        std_err: core::ptr::null_mut(),
        runtime_services: core::ptr::null_mut(),
        boot_services: core::ptr::null_mut(),
        number_of_table_entries: 0,
        configuration_table: core::ptr::null_mut(),
      };


fn dup_device_path(device_path: *mut c_void) -> *mut core::ffi::c_void{
    let mut device_path_buffer: *mut c_void = core::ptr::null_mut();
    let device_path_size = crate::efi::device_path::get_device_path_size (device_path as *mut DevicePathProtocol);
    log!("device_path_size: {:?}\n", device_path_size);
    let status = crate::efi::allocate_pool (MemoryType::BootServicesData, device_path_size, &mut device_path_buffer);
    unsafe {core::ptr::copy_nonoverlapping (device_path, device_path_buffer, device_path_size);}

    device_path_buffer
}


#[cfg(not(test))]
pub fn enter_uefi(hob: *const c_void) -> ! {

    unsafe {
      STDOUT.mode = &mut STDOUT_MODE;
      ST.con_in = &mut STDIN;
      ST.con_out = &mut STDOUT;
      ST.std_err = &mut STDOUT;
      ST.runtime_services = &mut RT;
      ST.boot_services = &mut BS;

      let func_addr_ptr = unsafe {transmute::<&mut InstallMultipleProtocolInterfacesFunc, *mut usize>(&mut BS.install_multiple_protocol_interfaces)};
      unsafe {*func_addr_ptr = install_multiple_protocol_interfaces_real as usize;}
      let func_addr_ptr = unsafe {transmute::<&mut UninstallMultipleProtocolInterfacesFunc, *mut usize>(&mut BS.uninstall_multiple_protocol_interfaces)};
      unsafe {*func_addr_ptr = uninstall_multiple_protocol_interfaces_real as usize;}

      ST.number_of_table_entries = MAX_CONFIGURATION_TABLE;
      ST.configuration_table = &mut CT as *mut [r_efi::system::ConfigurationTable; MAX_CONFIGURATION_TABLE] as *mut r_efi::system::ConfigurationTable;
    }

    crate::pi::hob_lib::dump_hob (hob);

    crate::efi::init::initialize_memory(hob);
    let new_hob = crate::pi::hob_lib::relocate_hob (hob);
    unsafe {
      CT[0].vendor_guid = crate::pi::hob::HOB_LIST_GUID;
      CT[0].vendor_table = new_hob;
    }

    unsafe {
      crate::efi::init::initialize_console (&mut ST, &mut STDIN_EX as *mut SimpleTextInputExProtocol as *mut c_void);
    }

    crate::efi::init::initialize_variable ();

    //crate::efi::init::initialize_fs ();

    pci::print_bus();

    let mut pci_transport;
    let mut device;
    let mut device_function;
    let mut device_device;
    match pci::search_bus(VIRTIO_PCI_VENDOR_ID, VIRTIO_PCI_BLOCK_DEVICE_ID) {
      Some(pci_device) => {
        device_function = pci_device.func;
        device_device = pci_device.device;
        pci_transport = pci::VirtioPciTransport::new(pci_device);
        device = crate::block::VirtioBlockDevice::new(&mut pci_transport);
        match device.init() {
            Err(_) => {
                log!("Error configuring block device search\n");
            }
            Ok(_) => log!(
                "Virtio block device configured. Capacity: {} sectors\n",
                device.get_capacity()
            ),
        }

        let mut f;
        let mut partition_start:u64;
        let mut partition_end:u64;

        match part::find_efi_partition(&device) {
            Ok((start, end)) => {
                log!("Found EFI partition\n");
                f = fat::Filesystem::new(&device, start, end);
                if f.init().is_err() {
                    log!("Failed to create filesystem\n");
                }
                partition_start = start;
                partition_end = end;
                let efi_part_id= unsafe { crate::efi::block::populate_block_wrappers(&mut crate::efi::BLOCK_WRAPPERS, &device) };
                log!("Filesystem ready\n");
                let mut wrapped_fs = file::FileSystemWrapper::new(&f, efi_part_id);
                let mut handle : Handle = core::ptr::null_mut();
                let status = crate::efi::install_protocol_interface (
                    &mut handle as *mut Handle,
                    &mut r_efi::protocols::simple_file_system::PROTOCOL_GUID as *mut Guid,
                    InterfaceType::NativeInterface,
                    &mut wrapped_fs.proto as *mut SimpleFileSystemProtocol as *mut c_void
                    );

                if status != Status::SUCCESS {
                    log!("Error");
                    }
                log!("simple_file_system protocol, handle: {:?}\n", handle);
                let mut file_system_path = HardDriveDevicePath {
                    file_system_path_node: HardDriveDevicePathNode {
                        header: DevicePathProtocol {
                        r#type: r_efi::protocols::device_path::TYPE_MEDIA,
                        sub_type: r_efi::protocols::device_path::Hardware::SUBTYPE_PCI,
                        length: [42, 0],
                        },
                        partition_number: efi_part_id.unwrap() as u32,
                        partition_start: partition_start as u64,
                        partition_size: partition_end - partition_start as u64,
                        partition_signature: [0x5452_4150_2049_4645u64,0],
                        partition_format: 0x2 as u8,
                        partition_type: 0x2 as u8,
                    },
                    end: r_efi::protocols::device_path::End {
                        header: DevicePathProtocol {
                        r#type: r_efi::protocols::device_path::TYPE_END,
                        sub_type: r_efi::protocols::device_path::End::SUBTYPE_ENTIRE,
                        length: [4, 0],
                        },
                    },
                };
                let mut device_path_buffer: *mut core::ffi::c_void = dup_device_path(&mut file_system_path.file_system_path_node.header as *mut DevicePathProtocol as *mut c_void);
                log!("device_path_buffer address: {:?}, device_path: {:?}\n", device_path_buffer, unsafe{*(device_path_buffer as *mut DevicePathProtocol)});
                let status = crate::efi::install_protocol_interface (
                        &mut handle,
                        &mut r_efi::protocols::device_path::PROTOCOL_GUID as *mut Guid,
                        InterfaceType::NativeInterface,
                        device_path_buffer
                        );

                let (image, size) = crate::efi::init::find_loader (new_hob);

                let mut image_path = FullMemoryMappedDevicePath {
                    memory_map: MemoryMappedDevicePathProtocol {
                        header: DevicePathProtocol {
                        r#type: r_efi::protocols::device_path::TYPE_HARDWARE,
                        sub_type: r_efi::protocols::device_path::Hardware::SUBTYPE_MMAP,
                        length: [24, 0],
                        },
                        memory_type: MemoryType::BootServicesCode,
                        start_address: image as u64,
                        end_address: image as u64 + size as u64 - 1,
                    },
                    end: r_efi::protocols::device_path::End {
                        header: DevicePathProtocol {
                        r#type: r_efi::protocols::device_path::TYPE_END,
                        sub_type: r_efi::protocols::device_path::End::SUBTYPE_ENTIRE,
                        length: [4, 0],
                        },
                    },
                };

                let mut image_handle : Handle = core::ptr::null_mut();
                let status = load_image (
                                Boolean::FALSE,
                                core::ptr::null_mut(), // parent handle
                                &mut image_path.memory_map.header as *mut DevicePathProtocol as *mut c_void,
                                image as *mut c_void,
                                size,
                                &mut image_handle
                                );
                match (status) {
                    Status::SUCCESS => {
                    let mut exit_data_size : usize = 0;
                    let mut exit_data : *mut Char16 = core::ptr::null_mut();
                    let status = start_image (
                                    image_handle,
                                    &mut exit_data_size as *mut usize,
                                    &mut exit_data as *mut *mut Char16
                                    );
                    },
                    _ => {
                    log!("load image fails {:?}\n", status);
                    },
                }
            }
            Err(_) => {
                log!("Failed to find EFI partition\n");
            }
        }
      },
      _ => {
      }
    }

    log!("Core Init Done\n");
    loop {}
}
