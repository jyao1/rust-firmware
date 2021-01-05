//! Block Io Protocol
//!
//! Provides the `open_volume` function returning a file protocol representing the root directory
//! of a filesystem.

pub const PROTOCOL_GUID: crate::base::Guid = crate::base::Guid::from_fields(
    0x964e5b21, 0x6459, 0x11d2, 0x8e, 0x39, &[0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b]
);

#[repr(C)]
pub struct BlockIoMedia {
    pub media_id: u32,
    pub removable_media: bool,
    pub media_present: bool,
    pub logical_partition: bool,
    pub read_only: bool,
    pub write_caching: bool,
    pub block_size: u32,
    pub io_align: u32,
    pub last_block: u64,
}

#[repr(C)]
pub struct Protocol {
    pub revision: u64,

    pub media: *const BlockIoMedia,

    pub reset: eficall! {fn(
        *mut Protocol,
        bool
    ) -> crate::efi::Status},

    pub read_blocks: eficall! {fn(
        *mut Protocol,
        u32,
        u64,
        usize,
        *mut core::ffi::c_void
    ) -> crate::efi::Status},

    pub write_blocks: eficall! {fn(
        *mut Protocol,
        u32,
        u64,
        usize,
        *mut core::ffi::c_void
    ) -> crate::efi::Status},

    pub flush_blocks: eficall! {fn(
        *mut Protocol,
    ) -> crate::efi::Status},
}
