//! Bundles the whole `assets/` directory into sharded asset archives.
//!
//! itch.io's HTML channel limits uploads to ~1000 files, but `assets/` contains
//! 25k+ files and rejects individual files over 200 MiB. Packing the assets into
//! 190 MiB shards keeps us under both limits while
//! the runtime [`PakAssetReader`](../asset_pak.rs) serves individual files back
//! out of the shards (via seek-read on native, HTTP range requests on wasm).
//!
//! Run with:
//!   cargo run --bin pack-assets                  (assets/ -> assets.pak)
//!   cargo run --bin pack-assets -- <src> <out>   (custom paths)
//!
//! ## Format (keep in sync with `src/asset_pak.rs`)
//! ```text
//! `assets.pak` contains `[ index ][ footer (24 bytes) ]`.
//! `assets-000.pak`, `assets-001.pak`, ... contain the raw asset data.
//! ```
//! Footer (last 24 bytes of the file):
//!   index_offset: u64 LE, index_length: u64 LE, magic b"ARCPAK02"
//! Index:
//!   entry_count: u32 LE, then per entry:
//!     path_len: u16 LE, path bytes (UTF-8, '/'-separated, relative to src root),
//!     shard: u16 LE, data_offset: u64 LE, data_length: u64 LE

use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Magic marker stored at the very end of the archive.
const MAGIC: &[u8; 8] = b"ARCPAK02";
/// Leave enough headroom below itch.io's 200 MiB per-file limit.
const SHARD_LIMIT: u64 = 190 * 1024 * 1024;

struct Entry {
    /// Forward-slash path relative to the source root (e.g. `images/icons/gold.ktx2`).
    path: String,
    shard: u16,
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

    for (abs_path, _) in &files {
        let file_length = abs_path.metadata()?.len();
        if file_length > SHARD_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} is {:.1} MiB; a single asset cannot exceed the {:.0} MiB shard limit",
                    abs_path.display(),
                    file_length as f64 / (1024.0 * 1024.0),
                    SHARD_LIMIT as f64 / (1024.0 * 1024.0),
                ),
            ));
        }
    }
    remove_old_shards(pak_path)?;

    let mut entries = Vec::with_capacity(files.len());
    let mut writer: Option<BufWriter<File>> = None;
    let mut shard: u16 = 0;
    let mut offset: u64 = 0;
    let mut buf = vec![0u8; 1 << 20];

    for (abs_path, rel_path) in &files {
        let file_length = abs_path.metadata()?.len();
        if offset > 0 && offset + file_length > SHARD_LIMIT {
            writer.take().expect("shard writer exists").flush()?;
            shard = shard.checked_add(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "asset archive needs too many shards")
            })?;
            offset = 0;
        }

        if writer.is_none() {
            writer = Some(BufWriter::new(File::create(shard_path(pak_path, shard))?));
        }

        let mut input = File::open(abs_path)?;
        let mut length: u64 = 0;
        loop {
            let read = input.read(&mut buf)?;
            if read == 0 {
                break;
            }
            writer.as_mut().expect("shard writer exists").write_all(&buf[..read])?;
            length += read as u64;
        }

        entries.push(Entry {
            path: rel_path.clone(),
            shard,
            offset,
            length,
        });
        offset += length;
    }
    if let Some(mut writer) = writer {
        writer.flush()?;
    }

    // The small root .pak is an index; asset bytes live in the numbered shards.
    let index = build_index(&entries);
    let out = File::create(pak_path)?;
    let mut writer = BufWriter::new(out);
    writer.write_all(&index)?;
    writer.write_all(&0u64.to_le_bytes())?;
    writer.write_all(&(index.len() as u64).to_le_bytes())?;
    writer.write_all(MAGIC)?;
    writer.flush()?;

    let shard_count = if entries.is_empty() {
        0
    } else {
        shard as usize + 1
    };
    println!(
        "Wrote {} entries across {} shards (max {:.0} MiB) plus index {}",
        entries.len(),
        shard_count,
        SHARD_LIMIT as f64 / (1024.0 * 1024.0),
        pak_path.display(),
    );
    Ok(())
}

fn shard_path(pak_path: &Path, shard: u16) -> PathBuf {
    let stem = pak_path.file_stem().and_then(|value| value.to_str()).unwrap_or("assets");
    pak_path.with_file_name(format!("{stem}-{shard:03}.pak"))
}

fn remove_old_shards(pak_path: &Path) -> io::Result<()> {
    let parent = pak_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = pak_path.file_stem().and_then(|value| value.to_str()).unwrap_or("assets");
    let prefix = format!("{stem}-");

    for entry in fs::read_dir(parent)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(number) = name.strip_prefix(&prefix).and_then(|value| value.strip_suffix(".pak"))
        else {
            continue;
        };
        if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn build_index(entries: &[Entry]) -> Vec<u8> {
    let mut index = Vec::new();
    index.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for entry in entries {
        let path_bytes = entry.path.as_bytes();
        index.extend_from_slice(&(path_bytes.len() as u16).to_le_bytes());
        index.extend_from_slice(path_bytes);
        index.extend_from_slice(&entry.shard.to_le_bytes());
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
