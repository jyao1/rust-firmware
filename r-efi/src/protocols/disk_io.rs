//! Disk Io Protocol
//!
//! Provides the `open_volume` function returning a file protocol representing the root directory
//! of a filesystem.

pub const PROTOCOL_GUID: crate::base::Guid = crate::base::Guid::from_fields(
    0xce345171, 0xba0b, 0x11d2, 0x8e, 0x4f, &[0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b]
);

pub const REVISION: u64 = 0x0000000000010000u64;

#[repr(C)]
pub struct Protocol {
    pub revision: u64,
    pub read_disk: eficall!{fn(
        *mut Protocol,
        u32,
        u64,
        usize,
        *mut core::ffi::c_void
    ) -> crate::base::Status},
    pub write_disk: eficall!{fn(
        *mut Protocol,
        u32,
        u64,
        usize,
        *mut core::ffi::c_void
    ) -> crate::base::Status},
}
