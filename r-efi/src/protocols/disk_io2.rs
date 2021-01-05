//! Disk Io Protocol
//!
//! Provides the `open_volume` function returning a file protocol representing the root directory
//! of a filesystem.

pub const PROTOCOL_GUID: crate::base::Guid = crate::base::Guid::from_fields(
    0x151c8eae, 0x7f2c, 0x472c, 0x9e, 0x54, &[0x98, 0x28, 0x19, 0x4f, 0x6a, 0x88]
);

pub const REVISION: u64 = 0x0000000000010000u64;

#[repr(C)]
pub struct DiskIo2Token {
    event: crate::efi::Event,
    transaction_status: crate::efi::Status
}

#[repr(C)]
pub struct Protocol {
    pub revision: u64,
    pub cancel: eficall!{fn(
        *mut Protocol
    )},

    pub read_disk_ex: eficall!{fn(
        *mut Protocol,
        u32,
        u64,
        usize,
        *mut core::ffi::c_void
    ) -> crate::base::Status},

    pub write_disk_ex: eficall!{fn(
        *mut Protocol,
        u32,
        u64,
        *mut DiskIo2Token,
        usize,
        *mut core::ffi::c_void
    ) -> crate::base::Status},

    pub flush_disk_ex: eficall!{fn(
        *mut Protocol,
        *mut DiskIo2Token
    ) -> crate::base::Status},
}
