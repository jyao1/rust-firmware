// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

use core::mem::size_of;
use r_efi::efi::Guid;
use r_uefi_pi::fv::{
    CommonSectionHeader, FfsFileHeader, FirmwareVolumeExtHeader, FirmwareVolumeHeader, FvBlockMap,
    FIRMWARE_FILE_SYSTEM2_GUID, FVH_SIGNATURE, FV_FILETYPE_DXE_CORE, FV_FILETYPE_FFS_PAD,
    FV_FILETYPE_RAW, FV_FILETYPE_SECURITY_CORE, SECTION_PE32, SECTION_RAW,
};

use scroll::{Pread, Pwrite};

use rust_firmware_layout::build_time::*;
#[allow(unused_imports)]
use rust_firmware_layout::consts::*;
use rust_fsp::fsp_t_upd::FsptUpd;

const RUST_VAR_AND_PADDING_SIZE: usize = (FIRMWARE_VAR_SIZE + FIRMWARE_PADDING_SIZE) as usize;
const RUST_PAYLOAD_MAX_SIZE: usize = FIRMWARE_PAYLOAD_SIZE as usize;
const RUST_IPL_MAX_SIZE: usize = FIRMWARE_IPL_SIZE as usize;
const RUST_RESET_VECTOR_MAX_SIZE: usize = FIRMWARE_RESET_VECTOR_SIZE as usize;

const RUST_PAYLOAD_OFFSET: usize = FIRMWARE_PAYLOAD_OFFSET as usize;
const RUST_IPL_OFFSET: usize = FIRMWARE_IPL_OFFSET as usize;

// this value is used by rust-ipl to find the firmware
const LOADED_IPL_ADDRESS: usize = LOADED_IPL_BASE as usize;

// size_of::<FirmwareVolumeHeader> = 56
// size_of::<FvBlockMap> = 8
// size_of::<FfsFileHeader> = 24
// size_of::<FirmwareVolumeExtHeader> = 20
// size_of::<PayloadFvHeader> = 120

#[repr(C)]
#[derive(Copy, Clone, Debug, Pwrite, Default)]
struct PayloadFvHeader {
    fv_header: FirmwareVolumeHeader,
    fv_block_map: [FvBlockMap; 2],
    pad_ffs_header: FfsFileHeader,
    fv_ext_header: FirmwareVolumeExtHeader,
    pad: [u8; 4],
}

#[derive(Copy, Clone, Debug, Pread, Pwrite, Default)]
struct PayloadFvFfsHeader {
    ffs_header: FfsFileHeader,
}

#[derive(Copy, Clone, Debug, Pread, Pwrite, Default)]
struct PayloadFvFfsSectionHeader {
    section_header: CommonSectionHeader,
}

#[repr(C, align(4))]
struct PayloadFvHeaderByte {
    data: [u8; size_of::<PayloadFvHeader>()
        + size_of::<PayloadFvFfsHeader>()
        + size_of::<PayloadFvFfsSectionHeader>()],
}
impl Default for PayloadFvHeaderByte {
    fn default() -> Self {
        PayloadFvHeaderByte {
            data: [0u8; size_of::<PayloadFvHeader>()
                + size_of::<PayloadFvFfsHeader>()
                + size_of::<PayloadFvFfsSectionHeader>()],
        }
    }
}

fn build_payload_fv_header(payload_fv_header_buffer: &mut [u8], payload_bin: &[u8]) {
    assert!(payload_bin.len() <= RUST_PAYLOAD_MAX_SIZE - size_of::<PayloadFvFfsHeader>());

    let mut payload_fv_header = PayloadFvHeader::default();

    let fv_header_size = (size_of::<PayloadFvHeader>()) as usize;

    payload_fv_header.fv_header.zero_vector = [0u8; 16];
    payload_fv_header
        .fv_header
        .file_system_guid
        .copy_from_slice(FIRMWARE_FILE_SYSTEM2_GUID.as_bytes());
    payload_fv_header.fv_header.fv_length = RUST_PAYLOAD_MAX_SIZE as u64;
    payload_fv_header.fv_header.signature = FVH_SIGNATURE;
    payload_fv_header.fv_header.attributes = 0x0004feff;
    payload_fv_header.fv_header.header_length = 0x0048;
    payload_fv_header.fv_header.checksum = 0x6403;
    payload_fv_header.fv_header.ext_header_offset = 0x0060;
    payload_fv_header.fv_header.reserved = 0x00;
    payload_fv_header.fv_header.revision = 0x02;

    payload_fv_header.fv_block_map[0].num_blocks = (RUST_PAYLOAD_MAX_SIZE as u32) / 0x1000;
    payload_fv_header.fv_block_map[0].length = 0x1000;
    payload_fv_header.fv_block_map[1].num_blocks = 0x0000;
    payload_fv_header.fv_block_map[1].length = 0x0000;

    payload_fv_header.pad_ffs_header.name.copy_from_slice(
        Guid::from_fields(
            0x00000000,
            0x0000,
            0x0000,
            0x00,
            0x00,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .as_bytes(),
    );
    payload_fv_header.pad_ffs_header.integrity_check = 0xaae4;
    payload_fv_header.pad_ffs_header.r#type = FV_FILETYPE_FFS_PAD;
    payload_fv_header.pad_ffs_header.attributes = 0x00;
    write_u24(0x2c, &mut payload_fv_header.pad_ffs_header.size);
    payload_fv_header.pad_ffs_header.state = 0x07u8;

    payload_fv_header.fv_ext_header.fv_name.copy_from_slice(
        Guid::from_fields(
            0x7cb8bdc9,
            0xf8eb,
            0x4f34,
            0xaa,
            0xea,
            &[0x3e, 0xe4, 0xaf, 0x65, 0x16, 0xa1],
        )
        .as_bytes(),
    );
    payload_fv_header.fv_ext_header.ext_header_size = 0x14;

    payload_fv_header.pad = [0u8; 4];

    let res1 = payload_fv_header_buffer
        .pwrite(payload_fv_header, 0)
        .unwrap();
    assert_eq!(res1, 120);

    let mut tdx_payload_fv_ffs_header = PayloadFvFfsHeader::default();
    tdx_payload_fv_ffs_header.ffs_header.name.copy_from_slice(
        //06948D4A-D359-4721-ADF6-5225485A6A3A
        Guid::from_fields(
            0x06948D4A,
            0xD359,
            0x4721,
            0xAD,
            0xF6,
            &[0x52, 0x25, 0x48, 0x5A, 0x6A, 0x3A],
        )
        .as_bytes(),
    );
    tdx_payload_fv_ffs_header.ffs_header.integrity_check = 0xaa4c;
    tdx_payload_fv_ffs_header.ffs_header.r#type = FV_FILETYPE_DXE_CORE;
    tdx_payload_fv_ffs_header.ffs_header.attributes = 0x00;
    write_u24(
        (payload_bin.len()
            + size_of::<PayloadFvFfsHeader>()
            + size_of::<PayloadFvFfsSectionHeader>()) as u32,
        &mut tdx_payload_fv_ffs_header.ffs_header.size,
    );
    tdx_payload_fv_ffs_header.ffs_header.state = 0xF8u8;

    let res2 = payload_fv_header_buffer
        .pwrite(tdx_payload_fv_ffs_header, fv_header_size)
        .unwrap();
    assert_eq!(res2, 24);

    let mut tdx_payload_fv_ffs_section_header = PayloadFvFfsSectionHeader::default();
    write_u24(
        (payload_bin.len() + size_of::<PayloadFvFfsSectionHeader>()) as u32,
        &mut tdx_payload_fv_ffs_section_header.section_header.size,
    );
    tdx_payload_fv_ffs_section_header.section_header.r#type = SECTION_PE32;

    let res3 = payload_fv_header_buffer
        .pwrite(
            tdx_payload_fv_ffs_section_header,
            fv_header_size + size_of::<PayloadFvFfsHeader>(),
        )
        .unwrap();
    assert_eq!(res3, 4);
}

type IplFvHeader = PayloadFvHeader;
type IplFvFfsHeader = PayloadFvFfsHeader;
type IplFvFfsSectionHeader = PayloadFvFfsSectionHeader;
type IplFvHeaderByte = PayloadFvHeaderByte;

fn build_ipl_fv_header(
    ipl_fv_header_buffer: &mut [u8],
    ipl_relocate_buffer: &[u8],
) {
    let mut ipl_fv_header = IplFvHeader::default();

    let fv_header_size = (size_of::<IplFvHeader>()) as usize;

    ipl_fv_header.fv_header.zero_vector = [0u8; 16];
    ipl_fv_header
        .fv_header
        .file_system_guid
        .copy_from_slice(FIRMWARE_FILE_SYSTEM2_GUID.as_bytes());

    // ipl_fv contains SecMain File and Volume Top File.
    ipl_fv_header.fv_header.fv_length = RUST_IPL_MAX_SIZE as u64;
    ipl_fv_header.fv_header.signature = FVH_SIGNATURE;
    ipl_fv_header.fv_header.attributes = 0x0004feff;
    ipl_fv_header.fv_header.header_length = 0x0048;
    ipl_fv_header.fv_header.checksum = 0xa528;
    ipl_fv_header.fv_header.ext_header_offset = 0x0060;
    ipl_fv_header.fv_header.reserved = 0x00;
    ipl_fv_header.fv_header.revision = 0x02;

    ipl_fv_header.fv_block_map[0].num_blocks =
        (RUST_IPL_MAX_SIZE / 0x1000) as u32;
    ipl_fv_header.fv_block_map[0].length = 0x1000;
    ipl_fv_header.fv_block_map[1].num_blocks = 0x0000;
    ipl_fv_header.fv_block_map[1].length = 0x0000;

    ipl_fv_header.pad_ffs_header.name.copy_from_slice(
        Guid::from_fields(
            0x00000000,
            0x0000,
            0x0000,
            0x00,
            0x00,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .as_bytes(),
    );
    ipl_fv_header.pad_ffs_header.integrity_check = 0xaae4;
    ipl_fv_header.pad_ffs_header.r#type = FV_FILETYPE_FFS_PAD;
    ipl_fv_header.pad_ffs_header.attributes = 0x00;
    write_u24(0x0, &mut ipl_fv_header.pad_ffs_header.size);
    ipl_fv_header.pad_ffs_header.state = 0x07u8;

    ipl_fv_header.fv_ext_header.fv_name.copy_from_slice(
        Guid::from_fields(
            0x763bed0d,
            0xde9f,
            0x48f5,
            0x81,
            0xf1,
            &[0x3e, 0x90, 0xe1, 0xb1, 0xa0, 0x15],
        )
        .as_bytes(),
    );

    ipl_fv_header.fv_ext_header.ext_header_size = 0x14;

    ipl_fv_header.pad = [0u8; 4];

    let _res = ipl_fv_header_buffer.pwrite(ipl_fv_header, 0).unwrap();

    let mut ipl_fv_ffs_header = IplFvFfsHeader::default();
    ipl_fv_ffs_header.ffs_header.name.copy_from_slice(
        // DF1CCEF6-F301-4A63-9661-FC6030DCC880
        Guid::from_fields(
            0xDF1CCEF6,
            0xF301,
            0x4A63,
            0x96,
            0x61,
            &[0xFC, 0x60, 0x30, 0xDC, 0xC8, 0x80],
        )
        .as_bytes(),
    );
    ipl_fv_ffs_header.ffs_header.integrity_check = 0xaacc;
    ipl_fv_ffs_header.ffs_header.r#type = FV_FILETYPE_SECURITY_CORE;
    ipl_fv_ffs_header.ffs_header.attributes = 0x00;
    write_u24(
        (ipl_relocate_buffer.len()
            + size_of::<IplFvFfsHeader>()
            + size_of::<IplFvFfsSectionHeader>()) as u32,
        &mut ipl_fv_ffs_header.ffs_header.size,
    );
    ipl_fv_ffs_header.ffs_header.state = 0xF8u8;

    let _res = ipl_fv_header_buffer
        .pwrite(ipl_fv_ffs_header, fv_header_size)
        .unwrap();

    let mut ipl_fv_ffs_section_header = IplFvFfsSectionHeader::default();
    write_u24(
        (ipl_relocate_buffer.len() + size_of::<IplFvFfsSectionHeader>()) as u32,
        &mut ipl_fv_ffs_section_header.section_header.size,
    );
    ipl_fv_ffs_section_header.section_header.r#type = SECTION_PE32;

    let _res = ipl_fv_header_buffer
        .pwrite(
            ipl_fv_ffs_section_header,
            fv_header_size + size_of::<IplFvFfsHeader>(),
        )
        .unwrap();
}

#[repr(C)]
#[derive(Debug, Default, Pwrite)]
struct ResetVectorHeader {
    ffs_header: FfsFileHeader,                        // 24
    section_header_pad: CommonSectionHeader,          // 4
    pad: [u8; 8],                                     // 8
    section_header_reset_vector: CommonSectionHeader, //4
}

const RESET_VECTOR_HEADER_SIZE: usize = size_of::<ResetVectorHeader>();
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
struct ResetVectorByte {
    data: [u8; RESET_VECTOR_HEADER_SIZE],
}
impl Default for ResetVectorByte {
    fn default() -> Self {
        ResetVectorByte {
            data: [0u8; RESET_VECTOR_HEADER_SIZE],
        }
    }
}

fn build_reset_vector_header(reset_vector_header_buffer: &mut [u8], reset_vector_bin: &[u8]) {
    let mut reset_vector_header = ResetVectorHeader::default();

    reset_vector_header.ffs_header.name.copy_from_slice(
        Guid::from_fields(
            0x1ba0062e,
            0xc779,
            0x4582,
            0x85,
            0x66,
            &[0x33, 0x6a, 0xe8, 0xf7, 0x8f, 0x09],
        )
        .as_bytes(),
    );
    reset_vector_header.ffs_header.integrity_check = 0xaa5a;
    reset_vector_header.ffs_header.r#type = FV_FILETYPE_RAW;
    reset_vector_header.ffs_header.attributes = 0x08;
    write_u24(
        (reset_vector_bin.len() + size_of::<ResetVectorHeader>()) as u32,
        &mut reset_vector_header.ffs_header.size,
    );
    reset_vector_header.ffs_header.state = 0x07u8;

    write_u24(0x0c, &mut reset_vector_header.section_header_pad.size);
    reset_vector_header.section_header_pad.r#type = SECTION_RAW;

    reset_vector_header.pad = [0u8; 8];

    write_u24(
        (reset_vector_bin.len() + size_of::<CommonSectionHeader>()) as u32,
        &mut reset_vector_header.section_header_reset_vector.size,
    );
    reset_vector_header.section_header_reset_vector.r#type = SECTION_RAW;

    let _res = reset_vector_header_buffer
        .pwrite(reset_vector_header, 0)
        .unwrap();
}

fn main() -> std::io::Result<()> {
    assert_eq!(
        RUST_VAR_AND_PADDING_SIZE
            + RUST_PAYLOAD_MAX_SIZE
            + RUST_IPL_MAX_SIZE
            + RUST_RESET_VECTOR_MAX_SIZE
            + FIRMWARE_FSP_T_SIZE as usize
            + FIRMWARE_FSP_M_SIZE as usize
            + FIRMWARE_FSP_S_SIZE as usize,
        FIRMWARE_SIZE as usize
    );
    assert!(RUST_PAYLOAD_MAX_SIZE > size_of::<PayloadFvHeader>());

    let args: Vec<String> = env::args().collect();

    let reset_vector_name = &args[1];
    let rust_ipl_name = &args[2];
    let rust_payload_name = &args[3];
    let rust_firmware_name = &args[4];

    let rust_fsp_type = if args.len() == 6 { &args[5] } else { "Qemu" };
    let (rust_fsp_t_bin, rust_fsp_m_bin, rust_fsp_s_bin) = match rust_fsp_type {
        "Qemu" => (
            include_bytes!("../../rust-fsp/fsp_bins/Qemu/Rebase/FspRel_T_FFFC5000.raw"),
            include_bytes!("../../rust-fsp/fsp_bins/Qemu/Rebase/FspRel_M_FFFC8000.raw"),
            include_bytes!("../../rust-fsp/fsp_bins/Qemu/Rebase/FspRel_S_FFFDD000.raw"),
        ),
        _ => {
            panic!("Must set to Qemu")
        }
    };
    let (fsp_t_bin, fsp_m_bin, fsp_s_bin) = (
        &rust_fsp_t_bin[..],
        &rust_fsp_m_bin[..],
        &rust_fsp_s_bin[..],
    );

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

    let zero_buf = vec![0xFFu8; FIRMWARE_SIZE as usize];

    let mut rust_payload_header_bytes = PayloadFvHeaderByte::default();

    let rust_payload_header_buffer = &mut rust_payload_header_bytes.data[..];
    build_payload_fv_header(rust_payload_header_buffer, rust_payload_bin.as_slice());

    let mut rust_ipl_header_bytes = IplFvHeaderByte::default();
    let rust_ipl_header_buffer = &mut rust_ipl_header_bytes.data[..];

    let mut new_rust_ipl_buf = vec![
        0x00u8;
        RUST_IPL_MAX_SIZE
            - size_of::<PayloadFvHeaderByte>()
            - size_of::<PayloadFvFfsSectionHeader>()
    ];
    let ipl_entry = pe_loader::pe::relocate(
        &rust_ipl_bin,
        &mut new_rust_ipl_buf,
        LOADED_IPL_ADDRESS + rust_ipl_header_buffer.len(),
    )
    .expect("fail to relocate PE image");
    let ipl_entry = ipl_entry as u32;

    build_ipl_fv_header(
        rust_ipl_header_buffer,
        new_rust_ipl_buf.as_slice(),
    );

    let mut rust_reset_vector_header_buffer = [0u8; size_of::<ResetVectorByte>()];
    build_reset_vector_header(
        &mut rust_reset_vector_header_buffer,
        reset_vector_bin.as_slice(),
    );

    let mut total_writen = 0usize;

    rust_firmware_file
        .write_all(&zero_buf[..RUST_PAYLOAD_OFFSET])
        .expect("fail to write pad");

    total_writen += &zero_buf[..RUST_PAYLOAD_OFFSET].len();

    rust_firmware_file
        .write_all(&rust_payload_header_buffer[..])
        .expect("fail to write rust payload header");
    total_writen += &rust_payload_header_buffer[..].len();

    rust_firmware_file
        .write_all(&rust_payload_bin[..])
        .expect("fail to write rust payload");
    total_writen += &rust_payload_bin[..].len();
    let pad_size =
        RUST_PAYLOAD_MAX_SIZE - rust_payload_bin.len() - rust_payload_header_buffer.len();
    rust_firmware_file
        .write_all(&zero_buf[..pad_size])
        .expect("fail to write pad");
    total_writen += &zero_buf[..pad_size].len();
    assert_eq!(total_writen, RUST_IPL_OFFSET);

    rust_firmware_file
        .write_all(&rust_ipl_header_buffer[..])
        .expect("fail to write rust IPL header");
    total_writen += &rust_ipl_header_buffer[..].len();
    rust_firmware_file
        .write_all(&new_rust_ipl_buf[..])
        .expect("fail to write rust IPL");
    total_writen += &new_rust_ipl_buf[..].len();

    let pad_size = RUST_IPL_MAX_SIZE - new_rust_ipl_buf.len() - rust_ipl_header_buffer.len();
    rust_firmware_file
        .write_all(&zero_buf[..pad_size])
        .expect("fail to write rust IPL");
    total_writen += &zero_buf[..pad_size].len();

    assert_eq!(total_writen, FIRMWARE_FSP_T_OFFSET as usize);

    rust_firmware_file
        .write_all(fsp_t_bin)
        .expect("fail to write rust fsp_t");
    assert_eq!(fsp_t_bin.len(), FIRMWARE_FSP_T_SIZE as usize);
    total_writen += fsp_t_bin.len();
    rust_firmware_file
        .write_all(fsp_m_bin)
        .expect("fail to write rust fsp_m");
    total_writen += fsp_m_bin.len();
    rust_firmware_file
        .write_all(fsp_s_bin)
        .expect("fail to write rust fsp_s");
    total_writen += fsp_s_bin.len();

    assert_eq!(total_writen, FIRMWARE_RESET_VECTOR_OFFSET as usize);
    // reset vector params
    #[derive(Debug, Pread, Pwrite)]
    struct ResetVectorParams {
        ipl_entry: u32,                 // rust ipl entry
        temp_ram_init_param: FsptUpd,   // FSP_T TempRamInit Params
    };

    let reset_vector_info = ResetVectorParams {
        ipl_entry,
        temp_ram_init_param: rust_firmware_qemu::fsp_data::TEMP_RAM_INIT_PARAM,
    };

    let reset_vector_info_buffer = &mut [0u8; 256];
    let writen = reset_vector_info_buffer
        .pwrite(reset_vector_info, 0)
        .unwrap();
    rust_firmware_file
        .write_all(&reset_vector_info_buffer[..writen])
        .expect("fail to write rust reset vector");

    let pad_size = RUST_RESET_VECTOR_MAX_SIZE
        - rust_reset_vector_header_buffer.len()
        - reset_vector_bin.len()
        - writen;
    rust_firmware_file
        .write_all(&zero_buf[..pad_size])
        .expect("fail to write rust reset vector");

    rust_firmware_file
        .write_all(&rust_reset_vector_header_buffer[..])
        .expect("fail to write reset vector header");

    rust_firmware_file
        .write_all(&reset_vector_bin[..])
        .expect("fail to write reset vector");

    rust_firmware_file.sync_data()?;

    Ok(())
}

fn write_u24(data: u32, buf: &mut [u8]) {
    assert_eq!(data < 0xffffff, true);
    buf[0] = (data & 0xFF) as u8;
    buf[1] = ((data >> 8) & 0xFF) as u8;
    buf[2] = ((data >> 16) & 0xFF) as u8;
}
