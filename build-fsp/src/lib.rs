// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

mod build_time_fsp_template;

use build_time_fsp_template::FspBuildTimeLayout;

const QEMU_FSP_RELEASE: &str = "./QemuFsp/BuildFsp/QEMU_FSP_RELEASE.fd";

pub struct FspGenerateParams {
    pub loaded_fsp_base: u32,
    pub firmware_fsp_offset: u32,
    pub firmware_fsp_max_size: u32,
}

pub fn generate_fsps(fsp_generate_params: &FspGenerateParams) {
    let fsp_split_names = split_fsp_binary().unwrap();
    let fsp_layout = generate_fsp_layout(&fsp_split_names, fsp_generate_params);
    let dest_path = Path::new("src").join("fsp_build_time.rs");
    generate_fsp_layout_file(&fsp_split_names, &fsp_layout, dest_path);
    rebase_fsp_binarys(&fsp_split_names, &fsp_layout);
}

struct FspSplitNames {
    // FSP-T file pathname
    pub fsp_t: PathBuf,
    // FSP-M file pathname
    pub fsp_m: PathBuf,
    // FSP-S file pathname
    pub fsp_s: PathBuf,
}

fn get_fsp_rebase_dir() -> PathBuf {
    let fsp_rebase_dir = PathBuf::from(
        std::env::var("OUT_DIR")
            .unwrap_or_else(|_| "Outputs".to_string())
            .as_str(),
    );
    if !fsp_rebase_dir.exists() {
        std::fs::create_dir_all(fsp_rebase_dir.clone()).expect("create dir failed");
    }
    fsp_rebase_dir
}

fn get_split_fsp_bin_py() -> PathBuf {
    let res = std::env::var("EDK2_PATH");
    let edk2_path = match res {
        Ok(edk2_path) => PathBuf::from(edk2_path),
        Err(_) => {
            let edk2_path = PathBuf::from("../QemuFsp");
            if !edk2_path.exists() {
                panic!("EDK2_PATH should be set!");
            }
            edk2_path
        }
    };

    if !edk2_path.exists() {
        panic!(
            "edk2 path {:?} not exist please set correct EDK2_PATH",
            edk2_path
        );
    }

    let split_fsp_bin_py = edk2_path
        .join("IntelFsp2Pkg")
        .join("Tools")
        .join("SplitFspBin.py");
    println!("{:?}", edk2_path);
    if !split_fsp_bin_py.exists() {
        panic!("{:?} not exists", split_fsp_bin_py);
    } else {
        println!("split_fsp_bin_py: {:?}", split_fsp_bin_py);
    }
    split_fsp_bin_py
}

fn split_fsp_binary() -> Option<FspSplitNames> {
    let fsp_binary_file_string =
        std::env::var("RUST_FIRMWARE_FSP_FD_FILE").unwrap_or_else(|_| -> String {
            println!(
                "RUST_FIRMWARE_FSP_FD_FILE should be set, use default {}",
                QEMU_FSP_RELEASE
            );
            QEMU_FSP_RELEASE.to_string()
        });

    let fsp_binary_filename = PathBuf::from(fsp_binary_file_string);

    if !fsp_binary_filename.exists() {
        println!("{:?} not exists", fsp_binary_filename);
    }
    let fsp_rebase_dir = get_fsp_rebase_dir();

    let split_fsp_bin_py = get_split_fsp_bin_py();

    let mut command = Command::new("python");

    // set outpug file name template is FSP.fv
    // output file will be FSP_S.fv FSP_M.fv FSP_T.fv
    command
        .arg(OsString::from(split_fsp_bin_py))
        .arg("split")
        .arg("-f")
        .arg(OsString::from(fsp_binary_filename))
        .arg("-o")
        .arg(OsString::from(fsp_rebase_dir))
        .arg("-n")
        .arg("FSP.fv");

    println!("command is {:?}", command);

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        print!("{}", s);
    } else {
        let s = String::from_utf8_lossy(&output.stderr);
        panic!("{}", s);
    }

    let fsp_rebase_dir = get_fsp_rebase_dir();

    let fsp_t = fsp_rebase_dir.join("FSP_T.fv");
    let fsp_m = fsp_rebase_dir.join("FSP_M.fv");
    let fsp_s = fsp_rebase_dir.join("FSP_S.fv");

    Some(FspSplitNames {
        fsp_t,
        fsp_m,
        fsp_s,
    })
}

fn generate_fsp_layout(
    fsp_split_names: &FspSplitNames,
    fsp_generate_params: &FspGenerateParams,
) -> FspBuildTimeLayout {
    let fsp_t_size = std::fs::metadata(fsp_split_names.fsp_t.to_owned())
        .unwrap_or_else(|_| panic!("cant open: {:?}", fsp_split_names.fsp_t))
        .len();
    let fsp_m_size = std::fs::metadata(fsp_split_names.fsp_m.to_owned())
        .expect("cant open")
        .len();
    let fsp_s_size = std::fs::metadata(fsp_split_names.fsp_s.to_owned())
        .expect("cant open")
        .len();

    let mut fsp_layout = FspBuildTimeLayout::new(
        fsp_generate_params.loaded_fsp_base,
        fsp_generate_params.firmware_fsp_offset,
        fsp_generate_params.firmware_fsp_max_size,
    );

    fsp_layout.update(fsp_t_size as u32, fsp_m_size as u32, fsp_s_size as u32);

    fsp_layout
}

fn generate_fsp_layout_file(
    fsp_names: &FspSplitNames,
    fsp_layout: &FspBuildTimeLayout,
    dest: PathBuf,
) {
    let context = format!(
        crate::BUILD_TIME_FSP_TEMPLATE!(),
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
        fsp_t_path = { get_rebase_filepathname(&fsp_names.fsp_t, "t") },
        fsp_m_path = { get_rebase_filepathname(&fsp_names.fsp_m, "m") },
        fsp_s_path = { get_rebase_filepathname(&fsp_names.fsp_s, "s") },
    );

    println!("dest: {:?}", dest);
    std::fs::write(dest, context).unwrap();
}

fn rebase_fsp_binarys(fsp_split_names: &FspSplitNames, fsp_layout: &FspBuildTimeLayout) {
    rebase_fsp_binary(&fsp_split_names.fsp_t, fsp_layout.fsp_t_base, "t");
    rebase_fsp_binary(&fsp_split_names.fsp_m, fsp_layout.fsp_m_base, "m");
    rebase_fsp_binary(&fsp_split_names.fsp_s, fsp_layout.fsp_s_base, "s");
}

fn get_rebase_filepathname(old: &PathBuf, typ: &'static str) -> PathBuf {
    let mut old = old.to_owned();
    old.set_file_name(format!("FSP_{}_REBASE.fv", typ.to_uppercase()));
    old
}

fn rebase_fsp_binary(f: &PathBuf, b: u32, c: &'static str) {
    let fsp_rebase_dir = get_fsp_rebase_dir();
    let split_fsp_bin_py = get_split_fsp_bin_py();

    let mut command = Command::new("python");

    let rebase_filepathname = get_rebase_filepathname(&f, c);

    println!("rebase name {:?}", rebase_filepathname);

    // set outpug file name template is FSP.fv
    // output file will be FSP_S.fv FSP_M.fv FSP_T.fv
    command
        .arg(OsString::from(split_fsp_bin_py))
        .arg("rebase")
        .arg("-f")
        .arg(OsString::from(f))
        .arg("-c")
        .arg(c)
        .arg("-b")
        .arg(format!("{:#X}", b))
        .arg("-o")
        .arg(fsp_rebase_dir)
        .arg("-n")
        .arg(rebase_filepathname);

    println!("command is {:?}", command);

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        print!("{}", s);
    } else {
        let s = String::from_utf8_lossy(&output.stderr);
        panic!("{}", s);
    }
}
