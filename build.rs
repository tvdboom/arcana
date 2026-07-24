//! Cargo build script for processing source assets and generating game catalogs.
//!
//! It also registers the asset inputs that should trigger a rebuild.

#[allow(dead_code)]
#[path = "src/bin/convert_to_ktx2.rs"]
mod convert_to_ktx2;

#[allow(dead_code)]
#[path = "src/bin/generate_catalogs.rs"]
mod catalog_gen;

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

/// Runs the build entry point.
fn main() {
    // Tell Cargo to rerun this build script if any assets or build files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bin/convert_to_ktx2.rs");
    println!("cargo:rerun-if-changed=src/bin/generate_catalogs.rs");
    rerun_if_changed_recursive(Path::new("assets-src"));

    let process_assets = cfg!(feature = "process-assets");
    let gen_catalogs = cfg!(feature = "generate-catalogs");

    if process_assets {
        println!("Processing assets: assets-src/ → assets/ (PNG → WebP)…");
        convert_to_ktx2::run("assets-src", "assets");
    } else {
        println!("Copying assets: assets-src/ → assets/ (no conversion)…");
        convert_to_ktx2::copy_only("assets-src", "assets");
    }

    if gen_catalogs {
        println!("Generating catalogs (img_ext=webp)…");
        catalog_gen::run("assets-src/images", "assets/catalog", "webp");
    }
}
