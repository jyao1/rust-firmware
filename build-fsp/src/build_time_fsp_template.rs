// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub struct FspBuildTimeLayout {
    pub fsp_base: u32,
    pub fsp_offset: u32,
    pub fsp_max_size: u32,
    pub fsp_max_base: u32,
    pub fsp_max_offset: u32,

    pub fsp_t_offset: u32,
    pub fsp_t_base: u32,
    pub fsp_t_size: u32,

    pub fsp_m_offset: u32,
    pub fsp_m_base: u32,
    pub fsp_m_size: u32,

    pub fsp_s_offset: u32,
    pub fsp_s_base: u32,
    pub fsp_s_size: u32,

    pub fsp_pad_size: u32,
}

impl FspBuildTimeLayout {
    pub fn new(fsp_base: u32, fsp_offset: u32, fsp_max_size: u32) -> FspBuildTimeLayout {
        FspBuildTimeLayout {
            fsp_base,
            fsp_offset,
            fsp_max_size,

            fsp_max_base: fsp_base + fsp_max_size,
            fsp_max_offset: fsp_offset + fsp_max_size,

            fsp_t_offset: 0,
            fsp_t_base: 0,
            fsp_t_size: 0,
            fsp_m_offset: 0,
            fsp_m_base: 0,
            fsp_m_size: 0,
            fsp_s_offset: 0,
            fsp_s_base: 0,
            fsp_s_size: 0,

            fsp_pad_size: 0,
        }
    }
    pub fn update(&mut self, fsp_t_size: u32, fsp_m_size: u32, fsp_s_size: u32) -> &mut Self {
        if self.fsp_max_size < fsp_t_size + fsp_m_size + fsp_s_size {
            panic!("fsp_max_size is too small, you should change it in rust-firmware-layout/etc/config.json");
        }

        self.fsp_t_size = fsp_t_size;
        self.fsp_m_size = fsp_m_size;
        self.fsp_s_size = fsp_s_size;

        self.fsp_pad_size = self.fsp_max_size - (fsp_t_size + fsp_m_size + fsp_s_size);

        let current_offset = self.fsp_offset;
        self.fsp_t_offset = current_offset;
        let current_offset = current_offset + self.fsp_t_size;
        self.fsp_m_offset = current_offset;
        let current_offset = current_offset + self.fsp_m_size;
        self.fsp_s_offset = current_offset;

        assert!(current_offset + self.fsp_s_size <= self.fsp_max_offset);

        let current_base = self.fsp_base;
        self.fsp_t_base = current_base;
        let current_base = current_base + self.fsp_t_size;
        self.fsp_m_base = current_base;
        let current_base = current_base + self.fsp_m_size;
        self.fsp_s_base = current_base;

        assert!(current_base + self.fsp_s_size <= self.fsp_max_base);

        self
    }
}

#[macro_export]
macro_rules! BUILD_TIME_FSP_TEMPLATE {
    () => {
        "// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/*
    FSP Layout:
                  Binary                       Address
            {fsp_t_offset:#010X} -> +--------------+ <- {fsp_t_base:#010X}
           ({fsp_t_size:#010X})   |  Rust Fsp-T  |
            {fsp_m_offset:#010X} -> +--------------+ <- {fsp_m_base:#010X}
           ({fsp_m_size:#010X})   |  Rust Fsp-M  |
            {fsp_s_offset:#010X} -> +--------------+ <- {fsp_s_base:#010X}
           ({fsp_s_size:#010X})   |  Rust Fsp-S  |
           ({fsp_pad_size:#010X})   |    (PAD)     |
            {fsp_max_offset:#010X} -> +--------------+ <- {fsp_max_base:#010X}
*/

// Image
pub const FIRMWARE_FSP_T_OFFSET: u32 = {fsp_t_offset:#X};
pub const FIRMWARE_FSP_M_OFFSET: u32 = {fsp_m_offset:#X};
pub const FIRMWARE_FSP_S_OFFSET: u32 = {fsp_s_offset:#X};
pub const FIRMWARE_FSP_MAX_OFFSET: u32 = {fsp_max_offset:#X};

pub const FIRMWARE_FSP_T_SIZE: u32 = {fsp_t_size:#X};
pub const FIRMWARE_FSP_M_SIZE: u32 = {fsp_m_size:#X};
pub const FIRMWARE_FSP_S_SIZE: u32 = {fsp_s_size:#X};
pub const FIRMWARE_FSP_PAD_SIZE: u32 = {fsp_pad_size:#X};

// Image loaded
pub const LOADED_FSP_T_BASE: u32 = {fsp_t_base:#X};
pub const LOADED_FSP_M_BASE: u32 = {fsp_m_base:#X};
pub const LOADED_FSP_S_BASE: u32 = {fsp_s_base:#X};
pub const LOADED_FSP_MAX_BASE: u32 = {fsp_max_base:#X};

// Image path
pub const FIRMWARE_FSP_T_PATH: &'static str = {fsp_t_path:?};
pub const FIRMWARE_FSP_M_PATH: &'static str = {fsp_m_path:?};
pub const FIRMWARE_FSP_S_PATH: &'static str = {fsp_s_path:?};
"
    };
}

#[test]
fn test_fsp_layout() {
    let mut fsp_layout = FspBuildTimeLayout::new(0xFFFC5000, 0x003C5000, 0x3A000);

    fsp_layout.update(0x00003000, 0x00022000, 0x00015000);

    let s = format!(
        BUILD_TIME_FSP_TEMPLATE!(),
        fsp_max_base = fsp_layout.fsp_max_base,
        fsp_max_offset = fsp_layout.fsp_max_offset,
        fsp_t_offset = fsp_layout.fsp_t_offset,
        fsp_t_size = fsp_layout.fsp_t_size,
        fsp_t_base = fsp_layout.fsp_t_base,
        fsp_m_offset = fsp_layout.fsp_m_offset,
        fsp_m_base = fsp_layout.fsp_m_base,
        fsp_m_size = fsp_layout.fsp_m_size,
        fsp_s_offset = fsp_layout.fsp_s_offset,
        fsp_s_base = fsp_layout.fsp_s_base,
        fsp_s_size = fsp_layout.fsp_s_size,
        fsp_pad_size = fsp_layout.fsp_pad_size,
        fsp_m_path = "Outputs/FSP_M_REBASE.fv",
        fsp_t_path = "Outputs/FSP_T_REBASE.fv",
        fsp_s_path = "Outputs/FSP_S_REBASE.fv",
    );
    println!("{}", s);
}
