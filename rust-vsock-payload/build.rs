use std::env;
 
fn main() {
    let out_dir = env::var("RUST_LINK_C_LIB_DIR").unwrap();
    let lib_name = env::var("RUST_LINK_C_LIB_NAME").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", lib_name);
}
