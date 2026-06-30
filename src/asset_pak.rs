//! Runtime [`AssetReader`] that serves individual files out of the single
//! `assets.pak` archive produced by the `pack-assets` binary.
//!
//! Shipping 25k+ loose asset files breaks itch.io's HTML channel (~1000 file
//! limit), so we bundle everything into one archive and read files back out:
//!   * native  -> open the file once and `seek` + `read` each requested range.
//!   * wasm    -> fetch the small index once, then issue an HTTP `Range` request
//!                per asset (the ~5 GB archive never lives in memory at once).
//!
//! ## Format (keep in sync with `src/bin/pack_assets.rs`)
//! ```text
//! [ data blob 0 ][ data blob 1 ] ... [ index ][ footer (24 bytes) ]
//! ```
//! Footer (last 24 bytes): index_offset:u64 LE, index_length:u64 LE, magic b"ARCPAK01".
//! Index: entry_count:u32 LE, then per entry:
//!   path_len:u16 LE, path bytes (UTF-8, '/'-separated), data_offset:u64 LE, data_length:u64 LE.

use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, AssetSourceId, PathStream, Reader, VecReader,
};
use bevy::asset::AssetApp;
use bevy::prelude::*;
use futures_lite::Stream;

/// Magic marker stored at the very end of the archive.
const MAGIC: &[u8; 8] = b"ARCPAK01";
/// Size of the fixed footer: index_offset (8) + index_length (8) + magic (8).
const FOOTER_LEN: usize = 24;
/// Default archive location (relative to the working dir / served alongside the wasm bundle).
const PAK_PATH: &str = "assets.pak";

/// Maps a normalized asset path to its `(offset, length)` within the archive.
type PakIndex = HashMap<String, (u64, u64)>;

/// Parses the 24-byte footer, returning `(index_offset, index_length)`.
fn parse_footer(footer: &[u8]) -> Option<(u64, u64)> {
    if footer.len() != FOOTER_LEN || &footer[16..24] != MAGIC {
        return None;
    }
    let index_offset = u64::from_le_bytes(footer[0..8].try_into().ok()?);
    let index_length = u64::from_le_bytes(footer[8..16].try_into().ok()?);
    Some((index_offset, index_length))
}

/// Parses the index section into a lookup table.
fn parse_index(bytes: &[u8]) -> Option<PakIndex> {
    let mut cursor = 0usize;
    let read = |cursor: &mut usize, n: usize| -> Option<&[u8]> {
        let slice = bytes.get(*cursor..*cursor + n)?;
        *cursor += n;
        Some(slice)
    };

    let count = u32::from_le_bytes(read(&mut cursor, 4)?.try_into().ok()?) as usize;
    let mut index = HashMap::with_capacity(count);
    for _ in 0..count {
        let path_len = u16::from_le_bytes(read(&mut cursor, 2)?.try_into().ok()?) as usize;
        let path = String::from_utf8(read(&mut cursor, path_len)?.to_vec()).ok()?;
        let offset = u64::from_le_bytes(read(&mut cursor, 8)?.try_into().ok()?);
        let length = u64::from_le_bytes(read(&mut cursor, 8)?.try_into().ok()?);
        index.insert(path, (offset, length));
    }
    Some(index)
}

/// Normalizes a Bevy asset path to the '/'-separated form used as the index key.
fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// A [`PathStream`] that yields nothing; we do not support directory listing.
struct EmptyPathStream;

impl Stream for EmptyPathStream {
    type Item = std::path::PathBuf;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}

/// Reads assets out of the bundled `assets.pak` archive.
pub struct PakAssetReader {
    inner: Arc<PakInner>,
}

impl PakAssetReader {
    async fn read_path<'a>(&'a self, path: &'a Path) -> Result<VecReader, AssetReaderError> {
        let key = normalize(path);
        self.inner.read_entry(&key, path).await
    }
}

impl AssetReader for PakAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.read_path(path).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // Meta files are not bundled (the app sets `AssetMetaCheck::Never`).
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Ok(Box::new(EmptyPathStream))
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

// --- Native backend: seek + read from the on-disk archive ------------------------------------

#[cfg(not(target_arch = "wasm32"))]
struct PakInner {
    pak_path: std::path::PathBuf,
    index: PakIndex,
}

#[cfg(not(target_arch = "wasm32"))]
impl PakInner {
    fn new_file(pak_path: std::path::PathBuf) -> std::io::Result<Self> {
        use std::io::{Read, Seek, SeekFrom};

        let mut file = std::fs::File::open(&pak_path)?;
        let len = file.metadata()?.len();
        if len < FOOTER_LEN as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "assets.pak is too small to be valid",
            ));
        }

        let mut footer = [0u8; FOOTER_LEN];
        file.seek(SeekFrom::End(-(FOOTER_LEN as i64)))?;
        file.read_exact(&mut footer)?;
        let (index_offset, index_length) = parse_footer(&footer).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "assets.pak footer is invalid")
        })?;

        let mut index_bytes = vec![0u8; index_length as usize];
        file.seek(SeekFrom::Start(index_offset))?;
        file.read_exact(&mut index_bytes)?;
        let index = parse_index(&index_bytes).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "assets.pak index is invalid")
        })?;

        Ok(Self { pak_path, index })
    }

    async fn read_entry(
        &self,
        key: &str,
        path: &Path,
    ) -> Result<VecReader, AssetReaderError> {
        use std::io::{Read, Seek, SeekFrom};

        let (offset, length) = *self
            .index
            .get(key)
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;

        if length == 0 {
            return Ok(VecReader::new(Vec::new()));
        }

        let mut file = std::fs::File::open(&self.pak_path).map_err(AssetReaderError::from)?;
        file.seek(SeekFrom::Start(offset)).map_err(AssetReaderError::from)?;
        let mut bytes = vec![0u8; length as usize];
        file.read_exact(&mut bytes).map_err(AssetReaderError::from)?;
        Ok(VecReader::new(bytes))
    }
}

// --- Wasm backend: HTTP range requests against the served archive ----------------------------

#[cfg(target_arch = "wasm32")]
struct PakInner {
    url: String,
    index: async_lock::OnceCell<PakIndex>,
}

#[cfg(target_arch = "wasm32")]
impl PakInner {
    fn new_http(url: String) -> Self {
        Self {
            url,
            index: async_lock::OnceCell::new(),
        }
    }

    async fn index(&self) -> Result<&PakIndex, AssetReaderError> {
        self.index
            .get_or_try_init(|| async {
                let footer = fetch_range(&self.url, RangeReq::Suffix(FOOTER_LEN as u64)).await?;
                let (index_offset, index_length) = parse_footer(&footer).ok_or_else(|| {
                    AssetReaderError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "assets.pak footer is invalid",
                        )
                        .into(),
                    )
                })?;
                let index_bytes =
                    fetch_range(&self.url, RangeReq::Range(index_offset, index_length)).await?;
                parse_index(&index_bytes).ok_or_else(|| {
                    AssetReaderError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "assets.pak index is invalid",
                        )
                        .into(),
                    )
                })
            })
            .await
    }

    async fn read_entry(
        &self,
        key: &str,
        path: &Path,
    ) -> Result<VecReader, AssetReaderError> {
        let (offset, length) = *self
            .index()
            .await?
            .get(key)
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;
        if length == 0 {
            return Ok(VecReader::new(Vec::new()));
        }
        let bytes = fetch_range(&self.url, RangeReq::Range(offset, length)).await?;
        Ok(VecReader::new(bytes))
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
enum RangeReq {
    /// Bytes `[start, start + len)`.
    Range(u64, u64),
    /// The final `len` bytes of the resource.
    Suffix(u64),
}

#[cfg(target_arch = "wasm32")]
fn js_err(context: &str, value: wasm_bindgen::JsValue) -> AssetReaderError {
    use js_sys::JSON;
    let message = JSON::stringify(&value)
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_else(|| "unknown error".to_string());
    AssetReaderError::Io(std::io::Error::other(format!("Failed to {context}: {message}")).into())
}

/// Fetches a byte range from `url`. Falls back to slicing a full `200` response if the
/// server ignores the `Range` header (so correctness never depends on range support).
#[cfg(target_arch = "wasm32")]
async fn fetch_range(url: &str, req: RangeReq) -> Result<Vec<u8>, AssetReaderError> {
    use js_sys::Uint8Array;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, Response};

    let header = match req {
        RangeReq::Range(start, len) => format!("bytes={}-{}", start, start + len.max(1) - 1),
        RangeReq::Suffix(len) => format!("bytes=-{len}"),
    };

    let request =
        Request::new_with_str(url).map_err(|e| js_err("create request", e))?;
    request
        .headers()
        .set("Range", &header)
        .map_err(|e| js_err("set Range header", e))?;

    // Resolve the global scope (window or worker) to call `fetch` on.
    let global = js_sys::global();
    let promise = if let Ok(window) = global.clone().dyn_into::<web_sys::Window>() {
        window.fetch_with_request(&request)
    } else if let Ok(worker) = global.dyn_into::<web_sys::WorkerGlobalScope>() {
        worker.fetch_with_request(&request)
    } else {
        return Err(AssetReaderError::Io(
            std::io::Error::other("Unsupported JavaScript global context").into(),
        ));
    };

    let resp_value = JsFuture::from(promise).await.map_err(|e| js_err("fetch path", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|e: JsValue| js_err("convert Response", e))?;

    match resp.status() {
        200 | 206 => {
            let buffer = JsFuture::from(
                resp.array_buffer().map_err(|e| js_err("read body", e))?,
            )
            .await
            .map_err(|e| js_err("await body", e))?;
            let all = Uint8Array::new(&buffer).to_vec();

            // 206 already contains exactly the requested range; 200 returns the whole
            // resource, so slice out the part we asked for.
            if resp.status() == 206 {
                Ok(all)
            } else {
                let bytes = match req {
                    RangeReq::Range(start, len) => {
                        let start = start as usize;
                        let end = (start + len as usize).min(all.len());
                        all.get(start..end).unwrap_or(&[]).to_vec()
                    }
                    RangeReq::Suffix(len) => {
                        let start = all.len().saturating_sub(len as usize);
                        all[start..].to_vec()
                    }
                };
                Ok(bytes)
            }
        }
        // itch.io's CDN returns 403 for missing files.
        403 | 404 => Err(AssetReaderError::NotFound(url.into())),
        status => Err(AssetReaderError::HttpError(status)),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use futures_lite::future::block_on;

    /// Writes a minimal archive in the on-disk format (mirrors `pack_assets.rs`).
    fn write_pak(path: &Path, files: &[(&str, &[u8])]) {
        use std::io::Write;

        let mut data = Vec::new();
        let mut index = Vec::new();
        index.extend_from_slice(&(files.len() as u32).to_le_bytes());
        for (name, bytes) in files {
            let offset = data.len() as u64;
            data.extend_from_slice(bytes);
            index.extend_from_slice(&(name.len() as u16).to_le_bytes());
            index.extend_from_slice(name.as_bytes());
            index.extend_from_slice(&offset.to_le_bytes());
            index.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        }
        let index_offset = data.len() as u64;
        data.extend_from_slice(&index);
        data.extend_from_slice(&index_offset.to_le_bytes());
        data.extend_from_slice(&(index.len() as u64).to_le_bytes());
        data.extend_from_slice(MAGIC);

        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&data).unwrap();
    }

    async fn read_all(inner: &PakInner, key: &str) -> Result<Vec<u8>, AssetReaderError> {
        let mut reader = inner.read_entry(key, Path::new(key)).await?;
        let mut out = Vec::new();
        Reader::read_to_end(&mut reader, &mut out).await.unwrap();
        Ok(out)
    }

    #[test]
    fn native_reader_round_trips() {
        let dir = std::env::temp_dir().join("arcana_pak_test");
        std::fs::create_dir_all(&dir).unwrap();
        let pak = dir.join("test.pak");

        let gold = b"hello-gold".to_vec();
        let music: Vec<u8> = (0u8..50).collect();
        write_pak(
            &pak,
            &[
                ("audio/music.ogg", &music),
                ("empty.bin", &[]),
                ("images/icons/gold.ktx2", &gold),
            ],
        );

        let inner = PakInner::new_file(pak).unwrap();

        assert_eq!(block_on(read_all(&inner, "images/icons/gold.ktx2")).unwrap(), gold);
        assert_eq!(block_on(read_all(&inner, "audio/music.ogg")).unwrap(), music);
        assert_eq!(block_on(read_all(&inner, "empty.bin")).unwrap(), Vec::<u8>::new());
        assert!(matches!(
            block_on(read_all(&inner, "missing.png")),
            Err(AssetReaderError::NotFound(_))
        ));
    }
}

/// Registers the `assets.pak` archive as the default asset source.
///
/// Must be called before `AssetPlugin` is added (i.e. before `DefaultPlugins`).
/// On native, if no archive is present we leave Bevy's default `assets/` folder
/// reader in place so iterative development keeps working without a pack step.
pub fn register(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    {
        let inner = Arc::new(PakInner::new_http(PAK_PATH.to_string()));
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || Box::new(PakAssetReader { inner: inner.clone() })),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let pak_path = std::path::PathBuf::from(PAK_PATH);
        if !pak_path.exists() {
            info!("'{PAK_PATH}' not found; loading assets from the 'assets/' folder.");
            return;
        }

        match PakInner::new_file(pak_path) {
            Ok(inner) => {
                let inner = Arc::new(inner);
                app.register_asset_source(
                    AssetSourceId::Default,
                    AssetSourceBuilder::new(move || {
                        Box::new(PakAssetReader { inner: inner.clone() })
                    }),
                );
            }
            Err(err) => {
                error!("Failed to open '{PAK_PATH}' ({err}); falling back to the 'assets/' folder.");
            }
        }
    }
}
