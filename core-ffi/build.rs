extern crate cbindgen;

use std::{env, path::PathBuf};
use cbindgen::Builder;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let output_path = crate_dir.join("../target/include/gg-ffi.hpp");

    Builder::new()
      .with_crate(crate_dir)
      .with_namespace("ffi")
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file(output_path);
}