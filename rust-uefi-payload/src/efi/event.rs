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

const EVENT_STRUCT_SIGNATURE: u32 = 0x54564549; // 'I','E','V','T'

#[derive(Default)]
struct EventStruct {
    signature: u32,
    r#type: u32,
    notify_tpl: Tpl,
    notify_function: usize,
    notify_context: usize,
}

const MAX_EVENT_STRUCT : usize = 16;

#[derive(Default)]
pub struct EventInfo {
    event_count: usize,
    event_struct: [EventStruct; MAX_EVENT_STRUCT],
}

impl EventInfo {
    pub fn create_event (
        &mut self,
        r#type: u32,
        notify_tpl: Tpl,
        notify_function: EventNotify,
        notify_context: *mut c_void,
    ) -> (Status, Event) {
        let (status, new_event) = self.get_new_event ();
        if status != Status::SUCCESS {
          return (status, core::ptr::null_mut());
        }

        let event_struct = unsafe {transmute::<Event, &mut EventStruct>(new_event)};
        event_struct.signature = EVENT_STRUCT_SIGNATURE;
        event_struct.r#type = r#type;
        event_struct.notify_tpl = notify_tpl;
        event_struct.notify_function = notify_function as usize;
        event_struct.notify_context = notify_context as usize;

        (Status::SUCCESS, new_event)
    }
    pub fn close_event (
        &mut self,
        event: Event
    ) -> (Status) {
        (Status::UNSUPPORTED)
    }
    fn get_new_event (
        &mut self
    ) -> (Status, Event) {
        if self.event_count >= MAX_EVENT_STRUCT {
          return (Status::OUT_OF_RESOURCES, core::ptr::null_mut());
        }
        let event_struct = &mut self.event_struct[self.event_count];
        self.event_count = self.event_count + 1;

        event_struct.signature = 0;

        (Status::SUCCESS, event_struct as *mut EventStruct as Event)
    }

    pub fn new() -> EventInfo {
        EventInfo {
            event_count: 0,
            ..EventInfo::default()
        }
    }
}

