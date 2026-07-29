//! Asset-processing pipeline that copies source assets and converts PNG images to WebP.
//!
//! The implementation is shared by its standalone binary and the build orchestrators.

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

const IMAGE_SETTINGS_VERSION: &str = "webp-q82-class-portrait-88-v2";
const CLASS_PORTRAIT_WIDTH_PERCENT: u32 = 88;

/// Performs the mtime operation.
fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

/// Returns whether update.
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

/// Collects files.
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

/// Removes stale images.
fn remove_stale_images(src_root: &Path, dst_root: &Path, keep_converted: bool) {
    let src_images = src_root.join("images");
    let dst_images = dst_root.join("images");
    for dst_path in collect_files(&dst_images) {
        let Ok(relative) = dst_path.strip_prefix(&dst_images) else {
            continue;
        };
        let extension = dst_path.extension().and_then(|ext| ext.to_str());
        let src_path = if matches!(extension, Some("ktx2" | "webp")) {
            src_images.join(relative).with_extension("png")
        } else {
            src_images.join(relative)
        };
        if extension == Some("zip")
            || (matches!(extension, Some("ktx2" | "webp")) && !keep_converted)
            || extension == Some("ktx2")
            || !src_path.exists()
        {
            if let Err(err) = fs::remove_file(&dst_path) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    panic!("remove stale asset {:?}: {err}", dst_path);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Crops a square class illustration to the playing-page portrait aspect ratio.
fn normalize_class_portrait(image: image::RgbaImage) -> image::RgbaImage {
    let (width, height) = image.dimensions();
    let target_width = height.saturating_mul(CLASS_PORTRAIT_WIDTH_PERCENT) / 100;
    if target_width == 0 || target_width >= width {
        return image;
    }

    image::imageops::crop_imm(&image, (width - target_width) / 2, 0, target_width, height)
        .to_image()
}

#[cfg(not(target_arch = "wasm32"))]
/// Converts one PNG, applying the standard crop to playable class portraits.
fn convert_single(src: &Path, dst: &Path, normalize_portrait: bool) {
    let mut image =
        image::open(src).unwrap_or_else(|err| panic!("decode {:?}: {err}", src)).into_rgba8();
    if normalize_portrait {
        image = normalize_class_portrait(image);
    }
    let (width, height) = image.dimensions();
    let encoded = webp::Encoder::from_rgba(image.as_raw(), width, height).encode(82.0);
    fs::write(dst, encoded.as_ref()).unwrap_or_else(|err| panic!("write WebP {:?}: {err}", dst));
}

#[cfg(target_arch = "wasm32")]
/// Converts single.
fn convert_single(_src: &Path, _dst: &Path, _normalize_portrait: bool) {
    panic!("asset conversion tools cannot run as WebAssembly");
}

enum AssetTask {
    Convert {
        src: PathBuf,
        dst: PathBuf,
        normalize_portrait: bool,
    },
    Copy {
        src: PathBuf,
        dst: PathBuf,
    },
}

/// Performs the log status operation.
fn log_status(msg: &str) {
    if std::env::var("OUT_DIR").is_ok() {
        println!("cargo:info={}", msg);
    } else {
        println!("{}", msg);
    }
}

/// Convert all assets: copy non-PNG files as-is, convert PNG → WebP.
/// Incremental: skips files where the destination is already up to date.
pub fn run(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);
    let settings_path = dst_root.join(".image-settings");
    if let Err(err) = fs::remove_file(dst_root.join(".ktx2-settings")) {
        if err.kind() != std::io::ErrorKind::NotFound {
            panic!("remove obsolete KTX2 settings file: {err}");
        }
    }
    let settings_changed = !matches!(
        fs::read_to_string(&settings_path),
        Ok(version) if version == IMAGE_SETTINGS_VERSION
    );

    let mut tasks = Vec::new();
    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        if ext == "zip" {
            continue;
        } else if ext == "png" {
            // Keep favicon.png as .png since the Windows window-icon loader needs it as PNG.
            let is_favicon = relative.to_string_lossy().contains("favicon.png");
            let dst_path = if is_favicon {
                dst_root.join(relative)
            } else {
                dst_root.join(relative).with_extension("webp")
            };
            if settings_changed || needs_update(&src_path, &dst_path) {
                if is_favicon {
                    tasks.push(AssetTask::Copy {
                        src: src_path,
                        dst: dst_path,
                    });
                } else {
                    let normalize_portrait = relative.parent() == Some(Path::new("images/classes"));
                    tasks.push(AssetTask::Convert {
                        src: src_path,
                        dst: dst_path,
                        normalize_portrait,
                    });
                }
            }
        } else {
            let dst_path = dst_root.join(relative);
            if needs_update(&src_path, &dst_path) {
                tasks.push(AssetTask::Copy {
                    src: src_path,
                    dst: dst_path,
                });
            }
        }
    }

    remove_stale_images(src_root, dst_root, true);

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

    let tasks = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let worker_count = std::thread::available_parallelism().map_or(1, usize::from).min(8);
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let tasks = tasks.clone();
            let pb = pb.clone();
            scope.spawn(move || loop {
                let task = tasks.lock().expect("asset task queue poisoned").pop_front();
                let Some(task) = task else {
                    break;
                };
                match task {
                    AssetTask::Convert {
                        src,
                        dst,
                        normalize_portrait,
                    } => {
                        let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
                        pb.set_message(format!("Converting: {}", name));
                        fs::create_dir_all(dst.parent().unwrap()).unwrap();
                        convert_single(&src, &dst, normalize_portrait);
                    },
                    AssetTask::Copy {
                        src,
                        dst,
                    } => {
                        let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
                        pb.set_message(format!("Copying: {}", name));
                        fs::create_dir_all(dst.parent().unwrap()).unwrap();
                        fs::copy(&src, &dst)
                            .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src, dst));
                    },
                }
                pb.inc(1);
            });
        }
    });
    pb.finish_with_message("Done!");
    fs::create_dir_all(dst_root).unwrap();
    fs::write(settings_path, IMAGE_SETTINGS_VERSION).unwrap();
}

/// Copy all assets as-is (no WebP conversion). Incremental.
pub fn copy_only(src_root: &str, dst_root: &str) {
    let src_root = Path::new(src_root);
    let dst_root = Path::new(dst_root);

    let mut tasks = Vec::new();
    for src_path in collect_files(src_root) {
        let relative = src_path.strip_prefix(src_root).expect("strip prefix");
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext == "zip" || (ext == "png" && !relative.to_string_lossy().contains("favicon.png")) {
            continue;
        }
        let dst_path = dst_root.join(relative);
        if needs_update(&src_path, &dst_path) {
            tasks.push(AssetTask::Copy {
                src: src_path,
                dst: dst_path,
            });
        }
    }

    remove_stale_images(src_root, dst_root, true);

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
        if let AssetTask::Copy {
            src,
            dst,
        } = task
        {
            let name = src.strip_prefix(src_root).unwrap_or(&src).to_string_lossy();
            pb.set_message(format!("Copying: {}", name));
            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {e}", src, dst));
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done!");
}

/// Runs the convert-to-ktx2 entry point.
fn main() {
    run("assets-src", "assets");
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    /// Verifies class portraits are center-cropped to the playing-page frame ratio.
    fn class_portrait_normalization_uses_standard_dimensions() {
        let source = RgbaImage::from_pixel(1_254, 1_254, Rgba([0, 0, 0, 255]));
        let normalized = normalize_class_portrait(source);

        assert_eq!(normalized.dimensions(), (1_103, 1_254));
    }
}
