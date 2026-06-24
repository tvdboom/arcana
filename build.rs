/// Orchestrator build binary.
///
/// Calls the same asset-processing and catalog-generation logic as the Cargo
/// build script, but as a standalone command:
///
///   cargo run --bin build-assets                 (uses feature flags compiled in)
///   cargo run --bin build-assets --no-default-features   (copy-only, no KTX2)
///
/// The root `build.rs` Cargo build script includes these same implementation
/// files so that `cargo build` also runs everything automatically.
#[allow(dead_code)]
mod convert_to_ktx2 {
    include!("src/bin/convert_to_ktx2.rs");
}

#[allow(dead_code)]
mod catalog_gen {
    include!("src/bin/generate_catalogs.rs");
}

use std::path::Path;

/// Recursively emit a `cargo:rerun-if-changed` line for `path` and, if it is a
/// directory, for every file and subdirectory it contains.
///
/// Cargo does NOT recurse into directories on its own: a bare
/// `rerun-if-changed=assets-src` only re-triggers when `assets-src`'s own mtime
/// changes, which does not happen when files are added/removed inside nested
/// subfolders. Walking the tree and registering each entry individually ensures
/// any add, remove, or modify anywhere under the directory re-runs the script.
fn rerun_if_changed_recursive(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                rerun_if_changed_recursive(&entry.path());
            }
        }
    }
}

fn main() {
    // Tell Cargo to rerun this build script if any assets or build files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bin/convert_to_ktx2.rs");
    println!("cargo:rerun-if-changed=src/bin/generate_catalogs.rs");
    rerun_if_changed_recursive(Path::new("assets-src"));

    let process_assets = cfg!(feature = "process-assets");
    let gen_catalogs = cfg!(feature = "generate-catalogs");

    if process_assets {
        println!("Processing assets: assets-src/ → assets/ (PNG → KTX2)…");
        convert_to_ktx2::run("assets-src", "assets");
    } else {
        println!("Copying assets: assets-src/ → assets/ (no conversion)…");
        convert_to_ktx2::copy_only("assets-src", "assets");
    }

    if gen_catalogs {
        println!("Generating catalogs (img_ext=ktx2…");
        catalog_gen::run("assets-src/images", "assets/catalog", "ktx2");
    }
}
