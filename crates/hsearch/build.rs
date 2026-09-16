use std::path::PathBuf;

fn main() {
    for path in ["../hsearch_core/src", "../hsearch_codegen/src"] {
        println!("cargo:rerun-if-changed={path}");
    }
    let out_dir = PathBuf::from("src/generated");
    let _ = std::fs::remove_dir_all(&out_dir); // ok if dir doesn't exist
    std::fs::create_dir_all(&out_dir).expect("failed to create dir for generated files");
    hsearch_codegen::generate_all(&out_dir).expect("failed to generate lookup tables");
}
