/// Asset-processing logic: copy assets-src/ → assets/, converting PNG → KTX2.
/// This file is used in two ways:
///   1. As the `convert_to_ktx2` binary  (`cargo run --bin convert_to_ktx2`)
///   2. Included via `include!()` in both `src/bin/build.rs` and the root `build.rs`

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn needs_update(src: &Path, dst: &Path) -> bool {
    match (mtime(src), mtime(dst)) {
        (Some(src_t), Some(dst_t)) => src_t > dst_t,
        (Some(_), None) => true,
        _ => false,
    }
}

fn collect_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path));
        } else if path.is_file() {
            files.push(path);
        }
    }
    files
}

fn convert_single(src: &Path, dst: &Path) {
    // toktx is part of KTX-Software: https://github.com/KhronosGroup/KTX-Software/releases
    // Install locally:  make install-ktx
    //
    // Uses zstd supercompression (--zcmp 18) which only requires the `ktx2` + `zstd_rust`
    // Bevy features (no C++ `basis-universal` dependency).
    // For GPU compression add `basis-universal` to Bevy features and change to:
    //   --encode uastc --uastc_quality 2 --zstd 18
    let status = Command::new("toktx")
        .args([
            "--zcmp",
            "18",
            "--genmipmap",
            "--assign_oetf",
            "srgb",
            "--assign_primaries",
            "bt709",
        ])
        .arg(dst)
        .arg(src)
        .status()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to run `toktx`. Please install KTX-Software:\n  \
                 https://github.com/KhronosGroup/KTX-Software/releases\n  \
                 or run: make install-ktx"
            )
        });

    if !status.success() {
        panic!("toktx failed: {:?} -> {:?}", src, dst);
    }
}

/// Convert all assets: copy non-PNG files as-is, convert PNG → KTX2.
/// Incremental: skips files where the destination is already up to date.
pub fn run(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);

    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "png" {
            let dst_path = dst_root.join(relative).with_extension("ktx2");
            if needs_update(&src_path, &dst_path) {
                fs::create_dir_all(dst_path.parent().unwrap()).unwrap();
                convert_single(&src_path, &dst_path);
            }
        } else {
            let dst_path = dst_root.join(relative);
            if needs_update(&src_path, &dst_path) {
                fs::create_dir_all(dst_path.parent().unwrap()).unwrap();
                fs::copy(&src_path, &dst_path)
                    .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src_path, dst_path));
            }
        }
    }
}

/// Copy all assets as-is (no KTX2 conversion). Incremental.
pub fn copy_only(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);

    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "png" {
            continue;
        }
        let dst_path = dst_root.join(relative);
        if needs_update(&src_path, &dst_path) {
            fs::create_dir_all(dst_path.parent().unwrap()).unwrap();
            fs::copy(&src_path, &dst_path)
                .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src_path, dst_path));
        }
    }
}

fn main() {
    run("assets-src", "assets");
}

