//! Bundles the whole `assets/` directory into a single `assets.pak` archive.
//!
//! itch.io's HTML channel limits uploads to ~1000 files, but `assets/` contains
//! 25k+ files. Packing everything into one file keeps us under that limit while
//! the runtime [`PakAssetReader`](../asset_pak.rs) serves individual files back
//! out of the archive (via seek-read on native, HTTP range requests on wasm).
//!
//! Run with:
//!   cargo run --bin pack-assets                  (assets/ -> assets.pak)
//!   cargo run --bin pack-assets -- <src> <out>   (custom paths)
//!
//! ## Format (keep in sync with `src/asset_pak.rs`)
//! ```text
//! [ data blob 0 ][ data blob 1 ] ... [ index ][ footer (24 bytes) ]
//! ```
//! Footer (last 24 bytes of the file):
//!   index_offset: u64 LE, index_length: u64 LE, magic b"ARCPAK01"
//! Index:
//!   entry_count: u32 LE, then per entry:
//!     path_len: u16 LE, path bytes (UTF-8, '/'-separated, relative to src root),
//!     data_offset: u64 LE, data_length: u64 LE

use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Magic marker stored at the very end of the archive.
const MAGIC: &[u8; 8] = b"ARCPAK01";

struct Entry {
    /// Forward-slash path relative to the source root (e.g. `images/icons/gold.ktx2`).
    path: String,
    offset: u64,
    length: u64,
}

/// Recursively collects every file under `root`, returning `(absolute_path, relative_path)`
/// pairs with the relative path normalized to forward slashes.
fn collect_files(root: &Path) -> io::Result<Vec<(PathBuf, String)>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .expect("entry is under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((path, rel));
            }
        }
    }

    // Deterministic ordering makes the archive reproducible.
    out.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(out)
}

/// Packs every file under `src_dir` into the single archive at `pak_path`.
pub fn run(src_dir: impl AsRef<Path>, pak_path: impl AsRef<Path>) {
    let src_dir = src_dir.as_ref();
    let pak_path = pak_path.as_ref();

    if let Err(err) = pack(src_dir, pak_path) {
        panic!("Failed to pack {} -> {}: {err}", src_dir.display(), pak_path.display());
    }
}

fn pack(src_dir: &Path, pak_path: &Path) -> io::Result<()> {
    let files = collect_files(src_dir)?;
    println!("Packing {} files from {} ...", files.len(), src_dir.display());

    let out = File::create(pak_path)?;
    let mut writer = BufWriter::new(out);

    let mut entries = Vec::with_capacity(files.len());
    let mut offset: u64 = 0;
    let mut buf = vec![0u8; 1 << 20];

    for (abs_path, rel_path) in &files {
        let mut input = File::open(abs_path)?;
        let mut length: u64 = 0;
        loop {
            let read = input.read(&mut buf)?;
            if read == 0 {
                break;
            }
            writer.write_all(&buf[..read])?;
            length += read as u64;
        }

        entries.push(Entry {
            path: rel_path.clone(),
            offset,
            length,
        });
        offset += length;
    }

    // `offset` now points just past the data section: the index starts here.
    let index_offset = offset;
    let index = build_index(&entries);
    writer.write_all(&index)?;

    // Footer: index_offset, index_length, magic.
    writer.write_all(&index_offset.to_le_bytes())?;
    writer.write_all(&(index.len() as u64).to_le_bytes())?;
    writer.write_all(MAGIC)?;

    writer.flush()?;

    let total = index_offset + index.len() as u64 + 24;
    println!(
        "Wrote {} ({:.1} MB, {} entries) -> {}",
        pak_path.display(),
        total as f64 / (1024.0 * 1024.0),
        entries.len(),
        pak_path.display(),
    );
    Ok(())
}

fn build_index(entries: &[Entry]) -> Vec<u8> {
    let mut index = Vec::new();
    index.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for entry in entries {
        let path_bytes = entry.path.as_bytes();
        index.extend_from_slice(&(path_bytes.len() as u16).to_le_bytes());
        index.extend_from_slice(path_bytes);
        index.extend_from_slice(&entry.offset.to_le_bytes());
        index.extend_from_slice(&entry.length.to_le_bytes());
    }
    index
}

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let src = args.get(1).map(String::as_str).unwrap_or("assets");
    let out = args.get(2).map(String::as_str).unwrap_or("assets.pak");
    run(src, out);
}
