use std::env;
use std::path::PathBuf;

fn main() {
    for path in ["../hsearch_core/src", "../hsearch_codegen/src"] {
        println!("cargo:rerun-if-changed={path}");
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    hsearch_codegen::generate_all(&out_dir).expect("failed to generate lookup tables");
}
