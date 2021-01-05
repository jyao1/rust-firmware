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

pub const PROTOCOL_GUID: crate::base::Guid = crate::base::Guid::from_fields(
    0xfd96974, 0x23aa, 0x4cdc, 0xb9, 0xcb, &[0x98, 0xd1, 0x77, 0x50, 0x32, 0x2a]
);

#[repr(C)]
pub struct Protocol {
    pub new_string: eficall!{fn(
        *mut Protocol,
        crate::efi::hii_database::HiiHandle,
        *mut crate::efi::hii::StringId,
        *mut r_efi::efi::Char8,
        *mut r_efi::efi::Char16,
        crate::efi::hii_font::String,
        *mut crate::efi::hii_font::FontInfo
    ) -> r_efi::efi::Status},
    pub get_string: eficall!{fn(
        *mut Protocol,
        *mut r_efi::efi::Char8,
        crate::efi::hii_database::HiiHandle,
        crate::efi::hii::StringId,
        crate::efi::hii_font::String,
        *mut usize,
        *mut *mut crate::efi::hii_font::FontInfo
    ) -> r_efi::efi::Status},
    pub set_string: eficall!{fn(
        *mut Protocol,
        crate::efi::hii_database::HiiHandle,
        crate::efi::hii::StringId,
        *mut r_efi::efi::Char8,
        crate::efi::hii_font::String,
        *mut crate::efi::hii_font::FontInfo
    ) -> r_efi::efi::Status},
    pub get_language: eficall!{fn(
        *mut Protocol,
        crate::efi::hii_database::HiiHandle,
        *mut r_efi::efi::Char8,
        *mut usize,
    ) -> r_efi::efi::Status},
    pub get_second_language: eficall!{fn(
        *mut Protocol,
        crate::efi::hii_database::HiiHandle,
        *mut r_efi::efi::Char8,
        *mut r_efi::efi::Char8,
        *mut usize,
    ) -> r_efi::efi::Status},
};