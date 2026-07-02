//! Cheap content signature used to prune size-collision groups before the
//! expensive full-file hash. Reads at most three 64 KB windows, never the
//! whole file, so it stays fast on large files.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const SAMPLE: u64 = 64 * 1024;

/// Fingerprints the first, middle, and last 64 KB (plus the size) of a file.
///
/// Two byte-identical files always share a fingerprint, so a difference here
/// proves the files differ and lets us skip full hashing. Distinct files may
/// still collide; the full hash stage resolves those, so this is only a filter.
pub fn fingerprint(path: &Path, size: u64) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; SAMPLE as usize];

    for offset in sample_offsets(size) {
        hash_window(&mut file, offset, &mut buf, &mut hasher)?;
    }
    hasher.update(&size.to_le_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

// First, middle, and tail windows. They may overlap on small files, which is
// harmless: the fingerprint stays deterministic and identical files still match.
fn sample_offsets(size: u64) -> [u64; 3] {
    [0, size / 2, size.saturating_sub(SAMPLE)]
}

fn hash_window(
    file: &mut File,
    offset: u64,
    buf: &mut [u8],
    hasher: &mut blake3::Hasher,
) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(offset))?;
    let mut filled = 0;
    while filled < buf.len() {
        let read = file.read(&mut buf[filled..])?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    hasher.update(&buf[..filled]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        File::create(&path).unwrap().write_all(bytes).unwrap();
        (dir, path)
    }

    #[test]
    fn identical_content_yields_identical_fingerprint() {
        let (_d1, a) = write(&vec![7u8; 300_000]);
        let (_d2, b) = write(&vec![7u8; 300_000]);
        assert_eq!(
            fingerprint(&a, 300_000).unwrap(),
            fingerprint(&b, 300_000).unwrap()
        );
    }

    #[test]
    fn differing_tail_changes_fingerprint() {
        let mut first = vec![7u8; 300_000];
        let mut second = first.clone();
        *first.last_mut().unwrap() = 1;
        *second.last_mut().unwrap() = 2;
        let (_d1, a) = write(&first);
        let (_d2, b) = write(&second);
        assert_ne!(
            fingerprint(&a, 300_000).unwrap(),
            fingerprint(&b, 300_000).unwrap()
        );
    }

    #[test]
    fn small_and_empty_files_fingerprint_without_error() {
        let (_d1, small) = write(b"hi");
        let (_d2, empty) = write(b"");
        assert!(fingerprint(&small, 2).is_ok());
        assert!(fingerprint(&empty, 0).is_ok());
    }
}
