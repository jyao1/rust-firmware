//! Driver Binding Protocol
//!
//! Provides the `open_volume` function returning a file protocol representing the root directory
//! of a filesystem.

pub const PROTOCOL_GUID: crate::base::Guid = crate::base::Guid::from_fields(
    0x18a031ab, 0xb443, 0x4d1a, 0xa5, 0xc0, &[0xc, 0x9, 0x26, 0x1e, 0x9f, 0x71]
);

#[repr(C)]
pub struct Protocol {
    pub supported: eficall!{fn(
        *mut Protocol,
        crate::efi::Handle,
        *mut crate::efi::protocols::device_path::Protocol
    ) -> crate::base::Status},
    pub start: eficall!{fn(
        *mut Protocol,
        crate::efi::Handle,
        *mut crate::efi::protocols::device_path::Protocol
    ) -> crate::base::Status},
    pub stop: eficall!{fn(
        *mut Protocol,
        crate::efi::Handle,
        usize,
        crate::efi::Handle
    ) -> crate::base::Status},
    pub version: u32,
    pub image_handle: crate::efi::Handle,
    pub driver_binding_handle: crate::efi::Handle
}
