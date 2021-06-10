pub mod protocols {
    pub mod device_path {
        use r_efi::protocols::device_path::Protocol;
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct Media {
            pub header: Protocol,
        }

        impl Media {
            pub const SUBTYPE_HARD_DRIVE: u8 = 0x1;
            pub const SUBTYPE_CD_ROM: u8 = 0x2;
            pub const SUBTYPE_VENDOR: u8 = 0x3;
            pub const SUBTYPE_FILE_PATH: u8 = 0x4;
            pub const SUBTYPE_MEDIA_PROTOCOL: u8 = 0x5;
            pub const SUBTYPE_PIWG_FIRMWARE_FILE: u8 = 0x6;
            pub const SUBTYPE_PIWG_FIRMWARE_VOLUMN: u8 = 0x7;
            pub const SUBTYPE_RELATIVE_OFFSET_RANGE: u8 = 0x9;
            pub const SUBTYPE_RAM_DISK: u8 = 0x9;
        }
    }

    pub mod file {

        // This file info is sized.
        #[repr(C)]
        #[derive(Debug)]
        pub struct Info {
            pub size: u64,
            pub file_size: u64,
            pub physical_size: u64,
            pub create_time: r_efi::system::Time,
            pub last_access_time: r_efi::system::Time,
            pub modification_time: r_efi::system::Time,
            pub attribute: u64,
        }
    }
}
