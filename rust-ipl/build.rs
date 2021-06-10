///
/// REF:https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
///

use std::{
    path::{Path, PathBuf},
    process::Command,
    env,
    fs,
};

// const RESET_VECTOR_SRC: &[(&str, &str)] = &[
//     ("x86_64", "ResetVector/Main.asm")
// ];

fn nasm(file: &Path, arch: &str, out_file: &Path, args: &[&str]) -> Command {
    let oformat = match arch {
        "x86_64" => ("win64"),
        "x86" => ("win32"),
        "bin" => ("bin"),
        _ => panic!("unsupported arch: {}", arch),
    };
    let mut c = Command::new("nasm");
    let _ = c
        .arg("-o")
        .arg(out_file.to_str().expect("Invalid path"))
        .arg("-f")
        .arg(oformat)
        .arg(file);
    for arg in args {
        let _ = c.arg(*arg);
    }
    c
}

fn run_command(mut cmd: Command) {
    eprintln!("running {:?}", cmd);
    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to execute [{:?}]: {}", cmd, e);
    });
    if !status.success() {
        panic!("execution failed");
    }
}

fn dump_env() {
    let env_list = &[
        "CARGO",
        "CARGO_MANIFEST_DIR",
        // "CARGO_MANIFEST_LINKS",
        "CARGO_MAKEFLAGS",
        "OUT_DIR",
        "TARGET",
        "CARGO_BIN_NAME",
        "CARGO_BUILD_TARGET"
        ];
    eprintln!("dump env: ");
    for v in env_list {
        eprintln!("{}: {}", *v, env::var(*v).unwrap_or_else(|_| "Not preset".to_string()));
    }
}


fn main() {
    dump_env();
    // tell cargo when to re-run the script
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", Path::new("ResetVector/ResetVector.asm").to_str().unwrap());

    let old_current_dir = env::current_dir().unwrap();
    let new_current_dir = old_current_dir.join("ResetVector");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = PathBuf::from(out_dir).join("ResetVector.bin");
    let copy_to_dir = out_file.parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap();
    let copy_to_file = copy_to_dir.join("ResetVector.bin");

    eprintln!("out_file is     {}", out_file.to_str().unwrap());
    eprintln!("copy_to_file is {}", copy_to_file.to_str().unwrap());

    let _ = env::set_current_dir(new_current_dir.as_path());
    run_command(nasm(Path::new("ResetVector.nasm"), "bin", out_file.as_path(), &[
        "-DARCH_X64",
        "-DPAGE_TABLE_SIZE=0x6000",
        "-DPAGE_TABLE_BASE=0x800000",
        "-DSEC_TOP_OF_STACK=0x830000",
        ]));
    let _ = env::set_current_dir(old_current_dir.as_path());
    let _ = fs::copy(&out_file, &copy_to_file).unwrap();
}
