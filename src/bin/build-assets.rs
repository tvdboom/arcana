//! Standalone asset-build orchestrator.
//!
//! It runs the asset conversion and catalog generation stages selected by features.

#[allow(dead_code)]
#[path = "convert_to_ktx2.rs"]
mod convert_to_ktx2;

#[allow(dead_code)]
#[path = "generate_catalogs.rs"]
mod catalog_gen;

/// Runs the build-assets entry point.
fn main() {
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
        let img_ext = if process_assets {
            "webp"
        } else {
            "png"
        };
        println!("Generating catalogs (img_ext={img_ext})…");
        catalog_gen::run("assets-src/images", "assets/catalog", img_ext);
    }
}
