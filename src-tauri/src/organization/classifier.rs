//! Maps a file to a category by extension, then MIME. Single source of classification rules.

use super::FileKind;
use crate::filesystem::FileEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationResult {
    pub kind: FileKind,
    pub reason: String,
}

// Extensions are stored lowercase by the scanner, so comparisons are exact.
const EXTENSION_TABLE: &[(FileKind, &[&str])] = &[
    (
        FileKind::Documents,
        &[
            "pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx", "md",
            "epub", "pages",
        ],
    ),
    (
        FileKind::Images,
        &[
            "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "heic", "tiff", "ico",
        ],
    ),
    (
        FileKind::Videos,
        &["mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "m4v"],
    ),
    (
        FileKind::Audio,
        &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma"],
    ),
    (
        FileKind::Archives,
        &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz"],
    ),
    (
        FileKind::Installers,
        &["dmg", "exe", "msi", "pkg", "deb", "rpm", "appimage", "apk"],
    ),
    (
        FileKind::Code,
        &[
            "rs", "ts", "tsx", "js", "jsx", "py", "java", "c", "cpp", "h", "go", "rb", "php", "sh",
            "json", "html", "css", "sql", "toml", "yaml", "yml",
        ],
    ),
];

pub fn classify(file: &FileEntry) -> ClassificationResult {
    if let Some(ext) = file.extension.as_deref() {
        if let Some(kind) = kind_for_extension(ext) {
            return ClassificationResult {
                kind,
                reason: format!(".{ext} file"),
            };
        }
    }
    if let Some(kind) = kind_for_mime(&file.mime_type) {
        return ClassificationResult {
            kind,
            reason: format!("Detected as {}", file.mime_type),
        };
    }
    ClassificationResult {
        kind: FileKind::Other,
        reason: "Unrecognized type".to_string(),
    }
}

fn kind_for_extension(ext: &str) -> Option<FileKind> {
    EXTENSION_TABLE
        .iter()
        .find(|(_, exts)| exts.contains(&ext))
        .map(|(kind, _)| *kind)
}

fn kind_for_mime(mime: &str) -> Option<FileKind> {
    if mime.starts_with("image/") {
        Some(FileKind::Images)
    } else if mime.starts_with("video/") {
        Some(FileKind::Videos)
    } else if mime.starts_with("audio/") {
        Some(FileKind::Audio)
    } else if mime.starts_with("text/") {
        Some(FileKind::Documents)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, ext: Option<&str>, mime: &str) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            extension: ext.map(str::to_string),
            path: format!("/dl/{name}"),
            size_bytes: 1,
            created_ms: None,
            modified_ms: None,
            mime_type: mime.to_string(),
            is_hidden: false,
        }
    }

    #[test]
    fn classifies_by_extension() {
        assert_eq!(
            classify(&file("a.png", Some("png"), "")).kind,
            FileKind::Images
        );
        assert_eq!(classify(&file("a.rs", Some("rs"), "")).kind, FileKind::Code);
        assert_eq!(
            classify(&file("a.zip", Some("zip"), "")).kind,
            FileKind::Archives
        );
    }

    #[test]
    fn falls_back_to_mime_when_extension_unknown() {
        assert_eq!(
            classify(&file("clip", None, "video/mp4")).kind,
            FileKind::Videos
        );
        assert_eq!(
            classify(&file("a.weirdext", Some("weirdext"), "image/png")).kind,
            FileKind::Images
        );
    }

    #[test]
    fn unknown_becomes_other() {
        assert_eq!(
            classify(&file("mystery", None, "application/octet-stream")).kind,
            FileKind::Other
        );
    }

    #[test]
    fn handles_unicode_names() {
        // Classification keys off extension/MIME, not the (here unicode) stem.
        assert_eq!(
            classify(&file("résumé café.pdf", Some("pdf"), "")).kind,
            FileKind::Documents
        );
    }
}
