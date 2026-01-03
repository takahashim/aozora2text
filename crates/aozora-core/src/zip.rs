//! ZIP ファイル処理
//!
//! CRC エラーを無視して ZIP ファイルを読み込む機能を提供します。
//! 青空文庫の一部の ZIP ファイルは CRC が不正なため、通常の方法では読み込めません。

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use flate2::read::DeflateDecoder;
use zip::CompressionMethod;

/// ZIP ファイルから最初の .txt ファイルを読み込む
///
/// CRC エラーを無視して読み込むため、CRC が不正な ZIP ファイルも処理できます。
///
/// # Examples
///
/// ```no_run
/// use aozora_core::zip::read_first_txt_from_zip;
/// use std::path::Path;
///
/// let content = read_first_txt_from_zip(Path::new("example.zip")).unwrap();
/// ```
pub fn read_first_txt_from_zip(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to read ZIP archive: {} ({})", e, path.display()),
        )
    })?;

    // .txt ファイルを探す
    for i in 0..archive.len() {
        let mut entry = archive.by_index_raw(i).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to read ZIP entry: {} ({})", e, path.display()),
            )
        })?;

        let entry_name = entry.name().to_string();
        let name = entry_name.to_lowercase();
        if name.ends_with(".txt") && !entry.is_dir() {
            return read_zip_entry_bytes(&mut entry, path, &entry_name);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("no .txt file found in ZIP archive: {}", path.display()),
    ))
}

/// ZIP エントリからバイト列を読み込む（CRC 検証をスキップ）
fn read_zip_entry_bytes(
    entry: &mut zip::read::ZipFile<'_>,
    path: &Path,
    entry_name: &str,
) -> io::Result<Vec<u8>> {
    if entry.encrypted() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "encrypted ZIP entry is not supported: {} ({})",
                entry_name,
                path.display()
            ),
        ));
    }

    let mut compressed = Vec::new();
    entry.read_to_end(&mut compressed).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "failed to read ZIP entry: {} ({} in {})",
                e,
                entry_name,
                path.display()
            ),
        )
    })?;

    match entry.compression() {
        CompressionMethod::Stored => Ok(compressed),
        CompressionMethod::Deflated => {
            let mut decoder = DeflateDecoder::new(&compressed[..]);
            let mut out = Vec::new();
            if entry.size() <= usize::MAX as u64 {
                out.reserve(entry.size() as usize);
            }
            decoder.read_to_end(&mut out).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!(
                        "failed to decompress ZIP entry: {} ({} in {})",
                        e,
                        entry_name,
                        path.display()
                    ),
                )
            })?;
            Ok(out)
        }
        method => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported ZIP compression method {:?}: {} ({})",
                method,
                entry_name,
                path.display()
            ),
        )),
    }
}

/// バイト列が ZIP ファイルかどうかをマジックバイトで判定
///
/// # Examples
///
/// ```
/// use aozora_core::zip::is_zip_file;
///
/// assert!(is_zip_file(b"PK\x03\x04test"));
/// assert!(!is_zip_file(b"not a zip"));
/// ```
pub fn is_zip_file(bytes: &[u8]) -> bool {
    // ZIPマジックバイト: PK\x03\x04 (通常) または PK\x05\x06 (空アーカイブ)
    bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zip_file() {
        assert!(is_zip_file(b"PK\x03\x04"));
        assert!(is_zip_file(b"PK\x05\x06"));
        assert!(is_zip_file(b"PK\x03\x04test content"));
        assert!(!is_zip_file(b"not a zip file"));
        assert!(!is_zip_file(b""));
        assert!(!is_zip_file(b"PK"));
    }
}
