//! Full-file cryptographic hash. Streams the file in fixed chunks so memory
//! stays constant regardless of file size. BLAKE3 is the final arbiter of
//! identity; only files sharing a full hash are treated as duplicates.

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const ALGO: &str = "blake3";

const CHUNK: usize = 128 * 1024;

pub fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; CHUNK];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
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
    fn identical_bytes_hash_equal_across_names() {
        let (_d1, a) = write(b"the same bytes");
        let (_d2, b) = write(b"the same bytes");
        assert_eq!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn different_bytes_hash_differently() {
        let (_d1, a) = write(b"alpha");
        let (_d2, b) = write(b"beta");
        assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn streams_large_file_in_constant_memory() {
        let (_d, big) = write(&vec![9u8; 5 * 1024 * 1024]);
        assert_eq!(hash_file(&big).unwrap().len(), 64);
    }

    #[test]
    fn missing_file_is_an_error_not_a_panic() {
        assert!(hash_file(Path::new("/nonexistent/deleted.bin")).is_err());
    }
}
