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
    include!("convert_to_ktx2.rs");
}

#[allow(dead_code)]
mod catalog_gen {
    include!("generate_catalogs.rs");
}

fn main() {
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
        let img_ext = if process_assets {
            "ktx2"
        } else {
            "png"
        };
        println!("Generating catalogs (img_ext={img_ext})…");
        catalog_gen::run("assets-src/images", "assets/inventory", img_ext);
    }
}
