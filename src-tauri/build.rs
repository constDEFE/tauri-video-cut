fn main() {
    tauri_build::build();

    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "debug" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = std::path::Path::new(&manifest_dir).join("lib");
    let dst = std::path::Path::new(&manifest_dir).join("target").join("debug");

    if src.exists() {
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), dst.join(entry.file_name())).ok();
        }
    }

    println!("cargo:rerun-if-changed=lib/");
}
