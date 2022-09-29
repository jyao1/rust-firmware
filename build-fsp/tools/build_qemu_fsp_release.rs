// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

//!
//! This toll will generate QEMU_FSP_RELEASE files
//! The generated location is at current_dir()
//!
//! This program required Git and Python installed
//! Also required EDK2 build environment. Goto
//! [Getting Started with EDK II](https://github.com/tianocore/tianocore.github.io/wiki/Getting-Started-with-EDK-II) to see more information.
//!
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const EDK2_PATH: &str = "QemuFsp";
const EDK2_GIT_PATH: &str = "https://github.com/tianocore/edk2.git";
const EDK2_VERSION: &str = "edk2-stable202011";
const QEMU_FSP_PATCH_URL: &str = "https://raw.githubusercontent.com/slimbootloader/slimbootloader/master/Silicon/QemuSocPkg/FspBin/Patches/0001-Build-QEMU-FSP-2.0-binaries.patch";
const QEMU_FSP_PATCH: &str = "0001-Build-QEMU-FSP-2.0-binaries.patch";
const QEMU_FSP_RELEASE_NAME: &str = "QEMU_FSP_RELEASE.fd";
const QEMU_FSP_RELEASE_DIR: &str = "BuildFsp";
fn main() {
    println!("build qemu fsp");
    let edk2_path = get_edk2_source_code_path();
    init_qemu_fsp_source_code(&edk2_path);
    build_qemu_fsp(&edk2_path);
}

///
/// Download edk2 source if it is not specify by EDK2_PATH.
///
fn get_edk2_source_code_path() -> PathBuf {
    // if environment is set
    let path = if std::env::var("EDK2_PATH").is_ok() {
        PathBuf::from(std::env::var("EDK2_PATH").unwrap())
    } else {
        download_edk2_source_code()
    };
    if !path.exists() {
        panic!("edk2 path not exist")
    }
    path
}

///
/// Download and apply QemuFspPkg path
///
fn init_qemu_fsp_source_code(edk2_path: &PathBuf) {
    let current_path = std::env::current_dir().expect("can't get current path");
    // pushd edk2
    std::env::set_current_dir(&edk2_path).expect("current dir set failed");

    let mut command = Command::new("curl");
    command
        .arg(QEMU_FSP_PATCH_URL)
        .arg("-o")
        .arg(QEMU_FSP_PATCH);
    run_command(&mut command);

    let mut command = Command::new("git");
    command
        .arg("am")
        .arg("--keep-cr")
        .arg("--whitespace=nowarn")
        .arg(QEMU_FSP_PATCH);
    run_command(&mut command);

    let mut command = Command::new("git");
    command.arg("submodule").arg("update").arg("--init");
    run_command(&mut command);
    // popd
    std::env::set_current_dir(&current_path).expect("current dir set failed");
}

///
/// Build QemuFsp
///
fn build_qemu_fsp(edk2_path: &PathBuf) -> PathBuf {
    let current_path = std::env::current_dir().expect("can't get current path");
    std::env::set_current_dir(&edk2_path).expect("current dir set failed");

    let mut command = Command::new("python");
    command.arg("BuildFsp.py").arg("/r");
    run_command(&mut command);

    let release_fsp_path = edk2_path.to_owned();
    let release_fsp_path = release_fsp_path
        .join(QEMU_FSP_RELEASE_DIR)
        .join(QEMU_FSP_RELEASE_NAME);

    std::env::set_current_dir(&current_path).expect("current dir set failed");

    release_fsp_path
}

fn download_edk2_source_code() -> PathBuf {
    let current_dir = std::env::current_dir().expect("get current dir failed");
    let edk2_path = current_dir.join(EDK2_PATH);
    if !edk2_path.exists() {
        let mut command = Command::new("git");
        command
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("-b")
            .arg(EDK2_VERSION)
            .arg(EDK2_GIT_PATH)
            .arg(EDK2_PATH);
        run_command(&mut command);
    }
    edk2_path
}

fn run_command(command: &mut Command) -> &mut Command {
    println!("command is {:?}", command);
    command.envs(std::env::vars());
    let child_out = command
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e))
        .stdout
        .unwrap_or_else(|| panic!("failed to get stdout"));

    let reader = BufReader::new(child_out);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    command
}
