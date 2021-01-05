// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![allow(unused)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

mod mem;
mod pe;

/*
    Image Layout:
                Binary                       Address
                   0 -> +--------------+ <-  0xFFC00000
                        |    Empty     |
            0x084000 -> +--------------+ <-  0xFFC84000
           (0x248000)   | Rust Payload |
                        |     (pad)    |
            0x2CC000 -> +--------------+ <-  0xFFECC000
           (0x133D20)   |   Rust IPL   |
                        |     (pad)    |
            0x3FFD20 -> +--------------+ <-  0xFFFFFD20
           (0x0002E0)   | Reset Vector |
       (4M) 0x400000 -> +--------------+ <- 0x100000000 (4G)
*/

const RUST_PAYLOAD_OFFSET: usize = 0x084000;
const RUST_PAYLOAD_SIZE: usize = 0x248000;
const RUST_IPL_OFFSET: usize = RUST_PAYLOAD_OFFSET + RUST_PAYLOAD_SIZE; // 0x2CC000;
const RUST_IPL_SIZE: usize = 0x133D20;
const RESET_VECTOR_OFFSET: usize = RUST_IPL_OFFSET + RUST_IPL_SIZE; // 0x3FFD20;
const RESET_VECTOR_SIZE: usize = 0x0002E0;
const RUST_FIRMWARE_SIZE: usize = RESET_VECTOR_OFFSET + RESET_VECTOR_SIZE; // 0x400000;

const RUST_FIRMWARE_BASE: usize = 0xFFFFFFFF - RUST_FIRMWARE_SIZE + 1;
const RUST_PAYLOAD_BASE: usize = RUST_FIRMWARE_BASE + RUST_PAYLOAD_OFFSET;
const RUST_IPL_BASE: usize = RUST_FIRMWARE_BASE + RUST_IPL_OFFSET;
const RESET_VECTOR_BASE: usize = RUST_FIRMWARE_BASE + RESET_VECTOR_OFFSET;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let reset_vector_name = &args[1];  
    let rust_ipl_name = &args[2];
    let rust_payload_name = &args[3];
    let rust_firmware_name = &args[4];

    println!(
        "\nrust-firmware-tool {} {} {} {}\n",
        reset_vector_name, rust_ipl_name, rust_payload_name, rust_firmware_name
    );

    let reset_vector_bin = fs::read(reset_vector_name).expect("fail to read reset_vector");
    //println!("{:?}", reset_vector_bin);
    let rust_ipl_bin = fs::read(rust_ipl_name).expect("fail to read rust IPL");
    let rust_payload_bin = fs::read(rust_payload_name).expect("fail to read rust payload");

    let mut rust_firmware_file =
        File::create(rust_firmware_name).expect("fail to create rust firmware");

    let zero_buf = vec![0xFFu8; RUST_FIRMWARE_SIZE];

    let rust_payload_header = vec![
        // FV header - Payload
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, // ZeroVector
        0x78u8, 0xe5u8, 0x8cu8, 0x8cu8, 0x3du8, 0x8au8, 0x1cu8, 0x4fu8, 0x99u8, 0x35u8, 0x89u8,
        0x61u8, 0x85u8, 0xc3u8, 0x2du8, 0xd3u8, // FileSystemGuid
        0x00u8, 0x80u8, 0x24u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, // FvLength
        0x5fu8, 0x46u8, 0x56u8, 0x48u8, // Signature
        0xffu8, 0xfeu8, 0x04u8, 0x00u8, // Attributes
        0x48u8, 0x00u8, // HeaderLength
        0x03u8, 0x64u8, // Checksum
        0x60u8, 0x00u8, // ExtHeaderOffset
        0x00u8, // Reserved
        0x02u8, // Revision
        0x48u8, 0x02u8, 0x00u8, 0x00u8, // BlockMap[0].NumBlocks
        0x00u8, 0x10u8, 0x00u8, 0x00u8, // BlockMap[0].Length
        0x00u8, 0x00u8, 0x00u8, 0x00u8, // BlockMap[1].NumBlocks
        0x00u8, 0x00u8, 0x00u8, 0x00u8, // BlockMap[1].Length
        // Pad FFS
        0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8,
        0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, // Name (GUID)
        0xf4u8, 0xaau8, //   IntegrityCheck
        0xf0u8, // Type
        0x00u8, // Attribute
        0x2cu8, 0x00u8, 0x00u8, // Size
        0xf8u8, // State
        // FV ExtHeader
        0xc9u8, 0xbdu8, 0xb8u8, 0x7cu8, 0xebu8, 0xf8u8, 0x34u8, 0x4fu8, 0xaau8, 0xeau8, 0x3eu8,
        0xe4u8, 0xafu8, 0x65u8, 0x16u8, 0xa1u8, // FvName
        0x14u8, 0x00u8, 0x00u8, 0x00u8, // ExtHeaderSize
        0xffu8, 0xffu8, 0xffu8, 0xffu8, // Pad
        // FFS header
        0x4au8, 0x8du8, 0x94u8, 0x06u8, 0x59u8, 0xd3u8, 0x21u8, 0x47u8, 0xadu8, 0xf6u8, 0x52u8,
        0x25u8, 0x48u8, 0x5au8, 0x6au8, 0x3au8, // Name (GUID)
        0x02u8, 0xaau8, // IntegrityCheck
        0x05u8, // Type - EFI_FV_FILETYPE_DXE_CORE
        0x00u8, // Attribute
        0x40u8, 0x52u8, 0x02u8, // Size
        0xf8u8, // State
        // Section Header
        0x0cu8, 0x52u8, 0x02u8, // Size
        0x10u8, // Type
    ];

    let rust_ipl_header = vec![
        // FV header - IPL
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, // ZeroVector
        0x78u8, 0xe5u8, 0x8cu8, 0x8cu8, 0x3du8, 0x8au8, 0x1cu8, 0x4fu8, 0x99u8, 0x35u8, 0x89u8,
        0x61u8, 0x85u8, 0xc3u8, 0x2du8, 0xd3u8, // FileSystemGuid
        0x00u8, 0x40u8, 0x13u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, // FvLength
        0x5fu8, 0x46u8, 0x56u8, 0x48u8, // Signature
        0xffu8, 0xfeu8, 0x04u8, 0x00u8, // Attributes
        0x48u8, 0x00u8, // HeaderLength
        0x28u8, 0xa5u8, // Checksum
        0x60u8, 0x00u8, // ExtHeaderOffset
        0x00u8, // Reserved
        0x02u8, // Revision
        0x34u8, 0x01u8, 0x00u8, 0x00u8, // BlockMap[0].NumBlocks
        0x00u8, 0x10u8, 0x00u8, 0x00u8, // BlockMap[0].Length
        0x00u8, 0x00u8, 0x00u8, 0x00u8, // BlockMap[1].NumBlocks
        0x00u8, 0x00u8, 0x00u8, 0x00u8, // BlockMap[1].Length
        // Pad FFS
        0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8,
        0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, // Name (GUID)
        0xf4u8, 0xaau8, // IntegrityCheck
        0xf0u8, // Type
        0x00u8, // Attribute
        0x2cu8, 0x00u8, 0x00u8, // Size
        0xf8u8, // State
        // FV ExtHeader
        0x0du8, 0xedu8, 0x3bu8, 0x76u8, 0x9fu8, 0xdeu8, 0xf5u8, 0x48u8, 0x81u8, 0xf1u8, 0x3eu8,
        0x90u8, 0xe1u8, 0xb1u8, 0xa0u8, 0x15u8, // FvName
        0x14u8, 0x00u8, 0x00u8, 0x00u8, // ExtHeaderSize
        0xffu8, 0xffu8, 0xffu8, 0xffu8, // Pad
        // FFS header
        0xf6u8, 0xceu8, 0x1cu8, 0xdfu8, 0x01u8, 0xf3u8, 0x63u8, 0x4au8, 0x96u8, 0x61u8, 0xfcu8,
        0x60u8, 0x30u8, 0xdcu8, 0xc8u8, 0x80u8, // Name (GUID)
        0xccu8, 0xaau8, // IntegrityCheck
        0x03u8, // Type - EFI_FV_FILETYPE_SECURITY_CORE
        0x00u8, // Attribute
        0x7eu8, 0x9au8, 0x12u8, // Size
        0xf8u8, // State
        // SECTION header
        0x44u8, 0x9au8, 0x12u8, // Size
        0x10u8, // Type
    ];

    let reset_vector_header = vec![
        // FFS header
        0x2eu8, 0x06u8, 0xa0u8, 0x1bu8, 0x79u8, 0xc7u8, 0x82u8, 0x45u8, 0x85u8, 0x66u8, 0x33u8,
        0x6au8, 0xe8u8, 0xf7u8, 0x8fu8, 0x09u8, // Name (GUID)
        0xf7u8, 0xaau8, // IntegrityCheck
        0x01u8, // Type - EFI_FV_FILETYPE_RAW
        0x08u8, // Attribute
        0x08u8, 0x03u8, 0x00u8, // Size, including FFS header 0x18 bytes.
        0xf8u8, // State
        // SECTION header - PAD
        0x0cu8, 0x00u8, 0x00u8, // Size, including section header 4 bytes.
        0x19u8, // Type - EFI_SECTION_RAW
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
        // SECTION header - ResecVector
        0xe4u8, 0x02u8, 0x00u8, // Size, including section header 4 bytes.
        0x19u8, // Type - EFI_SECTION_RAW
    ];

    let mut new_rust_ipl_buf = vec![0x00u8; RUST_IPL_SIZE - rust_ipl_header.len() - reset_vector_header.len()];
    pe::relocate(&rust_ipl_bin, &mut new_rust_ipl_buf, RUST_IPL_BASE + rust_ipl_header.len()).expect("fail to relocate PE image");

    rust_firmware_file
        .write_all(&zero_buf[..RUST_PAYLOAD_OFFSET])
        .expect("fail to write pad");

    rust_firmware_file
        .write_all(&rust_payload_header[..])
        .expect("fail to write rust payload header");
    rust_firmware_file
        .write_all(&rust_payload_bin[..])
        .expect("fail to write rust payload");
    let pad_size = RUST_PAYLOAD_SIZE - rust_payload_bin.len() - rust_payload_header.len();
    rust_firmware_file
        .write_all(&zero_buf[..pad_size])
        .expect("fail to write pad");

    rust_firmware_file
        .write_all(&rust_ipl_header[..])
        .expect("fail to write rust IPL header");
    rust_firmware_file
        .write_all(&new_rust_ipl_buf[..])
        .expect("fail to write rust IPL");

    rust_firmware_file
        .write_all(&reset_vector_header[..])
        .expect("fail to write reset vector header");

    rust_firmware_file
        .write_all(&reset_vector_bin[..])
        .expect("fail to write reset vector");

    rust_firmware_file.sync_data()?;

    Ok(())
}
