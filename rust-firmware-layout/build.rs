// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use serde::Deserialize;
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, fs::File};

macro_rules! BUILD_TIME_TEMPLATE {
    () => {
        "// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/*
    Image Layout:
                  Binary                       Address
            {reserved1_offset:#010X} -> +--------------+ <-  {reserved1_base:#010X}
            ({reserved1_size:#010X})  |  RESERVED1   |
            {padding_offset:#010X} -> +--------------+ <-  {padding_base:#010X}
            ({padding_size:#010X})  |   PADDING    |
            {payload_offset:#010X} -> +--------------+ <-  {payload_base:#010X}
           ({payload_size:#010X})   | Rust Payload |
                          |     (pad)    |
            {ipl_offset:#010X} -> +--------------+ <-  {ipl_base:#010X}
           ({ipl_size:#010X})   |   Rust IPL   |
                          |     (pad)    |
            {fsp_offset:#010X} -> +--------------+ <-  {fsp_base:#010X}
           ({fsp_max_size:#010X})   |   Rust Fsp   |
                          |     pad      |
            {reset_vector_offset:#010X} -> +--------------+ <-  {reset_vector_base:#010X}
                          | (DATA PATCH) |
           ({reset_vector_size:#010X})   | Reset Vector |
            {image_size:#010X} -> +--------------+ <- 0x100000000 (4G)
*/

// Image
pub const FIRMWARE_RESERVED1_OFFSET: u32 = {reserved1_offset:#X};
pub const FIRMWARE_PADDING_OFFSET: u32 = {padding_offset:#X};
pub const FIRMWARE_PAYLOAD_OFFSET: u32 = {payload_offset:#X};
pub const FIRMWARE_IPL_OFFSET: u32 = {ipl_offset:#X};
pub const FIRMWARE_FSP_OFFSET: u32 = {fsp_offset:#X};
pub const FIRMWARE_RESET_VECTOR_OFFSET: u32 = {reset_vector_offset:#X};

pub const FIRMWARE_SIZE: u32 = {image_size:#X};
pub const FIRMWARE_VAR_SIZE: u32 = {reserved1_size:#X};
pub const FIRMWARE_PADDING_SIZE: u32 = {padding_size:#X};
pub const FIRMWARE_PAYLOAD_SIZE: u32 = {payload_size:#X};
pub const FIRMWARE_IPL_SIZE: u32 = {ipl_size:#X};
pub const FIRMWARE_FSP_MAX_SIZE: u32 = {fsp_max_size:#X};
pub const FIRMWARE_RESET_VECTOR_SIZE: u32 = {reset_vector_size:#X};

// Image loaded
pub const LOADED_RESERVED1_BASE: u32 = {reserved1_base:#X};
pub const LOADED_PADDING_BASE: u32 = {padding_base:#X};
pub const LOADED_PAYLOAD_BASE: u32 = {payload_base:#X};
pub const LOADED_IPL_BASE: u32 = {ipl_base:#X};
pub const LOADED_FSP_BASE: u32 = {fsp_base:#X};
pub const LOADED_RESET_VECTOR_BASE: u32 = {reset_vector_base:#X};
"
    };
}

macro_rules! RUNTIME_TEMPLATE {
    () => {
        "// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/*
    Mem Layout:
                                            Address
                    +--------------+ <-  0x0
                    |     Legacy   |
                    +--------------+ <-  0x00100000 (1M)
                    |   ........   |
                    +--------------+
                    |   ........   |
                    +--------------+ <-  {heap_base:#010X}
                    |     HEAP     |    ({heap_size:#010X})
                    +--------------+ <-  {stack_base:#010X}
                    |     STACK    |    ({stack_size:#010X})
                    +--------------+ <-  {payload_base:#010X}
                    |    PAYLOAD   |    ({payload_size:#010X})
                    +--------------+ <-  {pt_base:#010X}
                    |  Page Table  |    ({pt_size:#010X})
                    +--------------+ <-  {hob_base:#010X}
                    |      HOB     |    ({hob_size:#010X})
                    +--------------+ <-  0x80000000 (2G) - for example
*/

pub const RUNTIME_HOB_SIZE: u32 = {hob_size:#X};
pub const RUNTIME_PAGE_TABLE_SIZE: u32 = {pt_size:#X};
pub const RUNTIME_PAYLOAD_SIZE: u32 = {payload_size:#X};
pub const RUNTIME_STACK_SIZE: u32 = {stack_size:#X};
pub const RUNTIME_HEAP_SIZE: u32 = {heap_size:#X};
"
    };
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
struct FirmwareLayoutConfig {
    image_layout: FirmwareImageLayoutConfig,
    runtime_layout: FirmwareRuntimeLayoutConfig,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
struct FirmwareImageLayoutConfig {
    image_size: u32,
    reserved1_size: u32,
    padding_size: u32,
    payload_size: u32,
    ipl_size: u32,
    fsp_max_size: u32,
    reset_vector_size: u32,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
struct FirmwareRuntimeLayoutConfig {
    hob_size: u32,
    stack_size: u32,
    heap_size: u32,
    payload_size: u32,
    page_table_size: u32,
}

#[derive(Debug, PartialEq)]
struct FirmwareLayout {
    config: FirmwareLayoutConfig,
    img: FirmwareLayoutImage,
    img_loaded: FirmwareLayoutImageLoaded,
    runtime: FirmwareLayoutRuntime,
}

impl FirmwareLayout {
    fn new_from_config(config: &FirmwareLayoutConfig) -> Self {
        let img = FirmwareLayoutImage::new_from_config(config);
        let img_loaded = FirmwareLayoutImageLoaded::new_from_image(config);

        FirmwareLayout {
            config: config.clone(),
            img,
            img_loaded,
            runtime: FirmwareLayoutRuntime::new_from_config(config),
        }
    }

    fn generate_build_time_rs(&self) {
        let mut to_generate = Vec::new();
        write!(
            &mut to_generate,
            BUILD_TIME_TEMPLATE!(),
            reserved1_size = self.config.image_layout.reserved1_size,
            padding_size = self.config.image_layout.padding_size,
            payload_size = self.config.image_layout.payload_size,
            ipl_size = self.config.image_layout.ipl_size,
            fsp_max_size = self.config.image_layout.fsp_max_size,
            reset_vector_size = self.config.image_layout.reset_vector_size,
            reserved1_offset = self.img.reserved1_offset,
            padding_offset = self.img.padding_offset,
            payload_offset = self.img.payload_offset,
            ipl_offset = self.img.ipl_offset,
            fsp_offset = self.img.fsp_offset,
            reset_vector_offset = self.img.reset_vector_offset,
            reserved1_base = self.img_loaded.reserved1_base,
            padding_base = self.img_loaded.padding_base,
            payload_base = self.img_loaded.payload_base,
            ipl_base = self.img_loaded.ipl_base,
            fsp_base = self.img_loaded.fsp_base,
            reset_vector_base = self.img_loaded.reset_vector_base,
            image_size = self.config.image_layout.image_size,
        )
        .expect("Failed to generate configuration code from the template and JSON config");

        let dest_path =
            Path::new(FIRMWARE_LAYOUT_CONFIG_RS_OUT_DIR).join(FIRMWARE_LAYOUT_BUILD_TIME_RS_OUT);
        fs::write(&dest_path, to_generate).unwrap();
    }

    fn generate_runtime_rs(&self) {
        let mut to_generate = Vec::new();
        write!(
            &mut to_generate,
            RUNTIME_TEMPLATE!(),
            hob_base = self.runtime.hob_base,
            hob_size = self.config.runtime_layout.hob_size,
            pt_base = self.runtime.pt_base,
            pt_size = self.config.runtime_layout.page_table_size,
            payload_base = self.runtime.payload_base,
            payload_size = self.config.runtime_layout.payload_size,
            heap_base = self.runtime.heap_base,
            heap_size = self.config.runtime_layout.heap_size,
            stack_base = self.runtime.stack_base,
            stack_size = self.config.runtime_layout.stack_size,
        )
        .expect("Failed to generate configuration code from the template and JSON config");

        let dest_path =
            Path::new(FIRMWARE_LAYOUT_CONFIG_RS_OUT_DIR).join(FIRMWARE_LAYOUT_RUNTIME_RS_OUT);
        fs::write(&dest_path, to_generate).unwrap();
    }

    pub fn generate_fsps(&self) {
        let fsp_generate_params = build_fsp::FspGenerateParams {
            loaded_fsp_base: self.img_loaded.fsp_base,
            firmware_fsp_offset: self.img.fsp_offset,
            firmware_fsp_max_size: self.config.image_layout.fsp_max_size,
        };
        build_fsp::generate_fsps(&fsp_generate_params);
    }
}

#[derive(Debug, Default, PartialEq)]
struct FirmwareLayoutImage {
    reserved1_offset: u32,
    padding_offset: u32,
    payload_offset: u32,
    ipl_offset: u32,
    reset_vector_offset: u32,
    fsp_offset: u32,
}

impl FirmwareLayoutImage {
    fn new_from_config(config: &FirmwareLayoutConfig) -> Self {
        let current_size = 0x0;
        let reserved1_offset = current_size;

        let current_size = current_size + config.image_layout.reserved1_size;
        let padding_offset = current_size;

        let current_size = current_size + config.image_layout.padding_size;
        let payload_offset = current_size;

        let current_size = current_size + config.image_layout.payload_size;
        let ipl_offset = current_size;

        let current_size = current_size + config.image_layout.ipl_size;
        let fsp_offset = current_size;

        let current_size = current_size + config.image_layout.fsp_max_size;
        let reset_vector_offset = current_size;

        FirmwareLayoutImage {
            reserved1_offset,
            padding_offset,
            payload_offset,
            ipl_offset,
            reset_vector_offset,
            fsp_offset,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct FirmwareLayoutImageLoaded {
    reserved1_base: u32,
    padding_base: u32,
    payload_base: u32,
    ipl_base: u32,
    fsp_base: u32,
    reset_vector_base: u32,
}

impl FirmwareLayoutImageLoaded {
    fn new_from_image(config: &FirmwareLayoutConfig) -> Self {
        let firmware_base = 0xFFFFFFFF - config.image_layout.image_size + 1;
        let reserved1_base = firmware_base;

        let current_base = reserved1_base + config.image_layout.reserved1_size;
        let padding_base = current_base;

        let current_base = current_base + config.image_layout.padding_size;
        let payload_base = current_base;

        let current_base = current_base + config.image_layout.payload_size;
        let ipl_base = current_base;

        let current_base = current_base + config.image_layout.ipl_size;
        let fsp_base = current_base;

        let current_base = current_base + config.image_layout.fsp_max_size;
        let reset_vector_base = current_base;

        FirmwareLayoutImageLoaded {
            reserved1_base,
            padding_base,
            payload_base,
            ipl_base,
            reset_vector_base,
            fsp_base,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct FirmwareLayoutRuntime {
    hob_base: u32,
    pt_base: u32,
    payload_base: u32,
    stack_base: u32,
    heap_base: u32,
}

impl FirmwareLayoutRuntime {
    fn new_from_config(config: &FirmwareLayoutConfig) -> Self {
        // TBD: assume LOW_MEM_TOP, to remove;
        const LOW_MEM_TOP: u32 = 0x80000000;
        let hob_base = LOW_MEM_TOP - config.runtime_layout.hob_size;
        let current_base = hob_base;

        let current_base = current_base - config.runtime_layout.page_table_size;
        let pt_base = current_base;

        let current_base = current_base - config.runtime_layout.payload_size;
        let payload_base = current_base;

        let current_base = current_base - config.runtime_layout.stack_size;
        let stack_base = current_base;

        let current_base = current_base - config.runtime_layout.heap_size;
        let heap_base = current_base;

        FirmwareLayoutRuntime {
            hob_base,
            pt_base,
            payload_base,
            stack_base,
            heap_base,
        }
    }
}

const FIRMWARE_LAYOUT_CONFIG_ENV: &str = "FIRMWARE_LAYOUT_CONFIG";
const FIRMWARE_LAYOUT_CONFIG_JSON_DEFAULT_PATH: &str = "etc/config.json";
const FIRMWARE_LAYOUT_CONFIG_RS_OUT_DIR: &str = "src";
const FIRMWARE_LAYOUT_BUILD_TIME_RS_OUT: &str = "build_time.rs";
const FIRMWARE_LAYOUT_RUNTIME_RS_OUT: &str = "runtime.rs";

fn main() {
    // Read and parse the Firmware layout configuration file.
    let mut data = String::new();
    let firmware_layout_config_json_file_path = env::var(FIRMWARE_LAYOUT_CONFIG_ENV)
        .unwrap_or_else(|_| FIRMWARE_LAYOUT_CONFIG_JSON_DEFAULT_PATH.to_string());
    let mut firmware_layout_config_json_file = File::open(firmware_layout_config_json_file_path)
        .expect("The Firmware layout configuration file does not exist");
    firmware_layout_config_json_file
        .read_to_string(&mut data)
        .expect("Unable to read string");
    let firmware_layout_config: FirmwareLayoutConfig =
        json5::from_str(&data).expect("It is not a valid Firmware layout configuration file.");

    let layout = FirmwareLayout::new_from_config(&firmware_layout_config);
    // TODO: sanity checks on the layouts.

    // Generate config .rs file from the template and JSON inputs, then write to fs.
    layout.generate_build_time_rs();
    layout.generate_runtime_rs();
    // layout.generate_fsps();

    // Re-run the build script if the files at the given paths or envs have changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../Cargo.lock");
    println!(
        "cargo:rerun-if-changed={}",
        FIRMWARE_LAYOUT_CONFIG_JSON_DEFAULT_PATH
    );
    println!("cargo:rerun-if-env-changed={}", FIRMWARE_LAYOUT_CONFIG_ENV);
}
