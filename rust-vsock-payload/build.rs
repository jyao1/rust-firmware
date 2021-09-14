use std::{env, str::FromStr};
 
fn main() {
    let out_dir = match env::var("RUST_LINK_C_LIB_DIR") {
        Ok(dir) => dir,
        Err(_e) => String::from_str("rust-vsock-payload/").unwrap()
    };
    let lib_name = match env::var("RUST_LINK_C_LIB_NAME") {
        Ok(name) => name,
        Err(_e) => String::from_str("main").unwrap()
    };
    println!("cargo:rerun-if-env-changed={}", "RUST_LINK_C_LIB_DIR");
    println!("cargo:rerun-if-env-changed={}", "RUST_LINK_C_LIB_NAME");
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", lib_name);
}
