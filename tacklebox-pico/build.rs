use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` into the build output directory
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();

    // Tell cargo to pass `OUT_DIR` as a search path to the linker
    println!("cargo:rustc-link-search={}", out.display());

    // Re-run if `memory.x` changes
    println!("cargo:rerun-if-changed=memory.x");
}
