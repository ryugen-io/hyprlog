//! Build script for `hl_ffi` - generates C header via cbindgen.

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = PathBuf::from(&crate_dir)
        .join("..")
        .join("..")
        .join("include");

    // Create include directory if it doesn't exist
    std::fs::create_dir_all(&out_dir).ok();

    let config = cbindgen::Config::from_file(PathBuf::from(&crate_dir).join("cbindgen.toml"))
        .expect("Failed to read cbindgen.toml");

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(out_dir.join("hyprlog.h"));

    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=cbindgen.toml");
}
