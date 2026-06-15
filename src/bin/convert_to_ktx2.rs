/// Asset-processing logic: copy assets-src/ → assets/, converting PNG → KTX2.
/// This file is used in two ways:
///   1. As the `convert_to_ktx2` binary  (`cargo run --bin convert_to_ktx2`)
///   2. Included via `include!()` in both `src/bin/build.rs` and the root `build.rs`
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use indicatif::{ProgressBar, ProgressStyle};

fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn needs_update(src: &Path, dst: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(dst) {
        if metadata.len() == 0 {
            return true;
        }
    } else {
        return true;
    }

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

enum AssetTask {
    Convert { src: PathBuf, dst: PathBuf },
    Copy { src: PathBuf, dst: PathBuf },
}

fn log_status(msg: &str) {
    if std::env::var("OUT_DIR").is_ok() {
        println!("cargo:warning={}", msg);
    } else {
        println!("{}", msg);
    }
}

/// Convert all assets: copy non-PNG files as-is, convert PNG → KTX2.
/// Incremental: skips files where the destination is already up to date.
pub fn run(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);

    let mut tasks = Vec::new();
    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        if ext == "png" {
            let dst_path = dst_root.join(relative).with_extension("ktx2");
            if needs_update(&src_path, &dst_path) {
                tasks.push(AssetTask::Convert { src: src_path, dst: dst_path });
            }
        } else {
            let dst_path = dst_root.join(relative);
            if needs_update(&src_path, &dst_path) {
                tasks.push(AssetTask::Copy { src: src_path, dst: dst_path });
            }
        }
    }

    let total = tasks.len();
    if total == 0 {
        log_status("All assets are up to date.");
        return;
    }

    log_status(&format!("Processing {} asset updates...", total));

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .expect("valid template")
            .progress_chars("#>-")
    );

    for task in tasks.into_iter() {
        match task {
            AssetTask::Convert { src, dst } => {
                let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
                pb.set_message(format!("Converting: {}", name));
                fs::create_dir_all(dst.parent().unwrap()).unwrap();
                convert_single(&src, &dst);
            }
            AssetTask::Copy { src, dst } => {
                let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
                pb.set_message(format!("Copying: {}", name));
                fs::create_dir_all(dst.parent().unwrap()).unwrap();
                fs::copy(&src, &dst)
                    .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src, dst));
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done!");
}

/// Copy all assets as-is (no KTX2 conversion). Incremental.
pub fn copy_only(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);

    let mut tasks = Vec::new();
    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext == "png" {
            continue;
        }
        let dst_path = dst_root.join(relative);
        if needs_update(&src_path, &dst_path) {
            tasks.push(AssetTask::Copy { src: src_path, dst: dst_path });
        }
    }

    let total = tasks.len();
    if total == 0 {
        log_status("All non-PNG assets are up to date.");
        return;
    }

    log_status(&format!("Copying {} assets (no conversion)...", total));

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .expect("valid template")
            .progress_chars("#>-")
    );

    for task in tasks.into_iter() {
        match task {
            AssetTask::Copy { src, dst } => {
                let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
                pb.set_message(format!("Copying: {}", name));
                fs::create_dir_all(dst.parent().unwrap()).unwrap();
                fs::copy(&src, &dst)
                    .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src, dst));
            }
            _ => {}
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done!");
}

fn main() {
    run("assets-src", "assets");
}
