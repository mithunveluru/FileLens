//! Read-only rule engine: flags files and computes summary totals.

pub mod commands;

use std::collections::HashMap;

use serde::Serialize;

use crate::filesystem::FileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    LargeFile,
    OldFile,
    Installer,
    TemporaryFile,
    Duplicate,
}

const ALL_CATEGORIES: [Category; 5] = [
    Category::LargeFile,
    Category::OldFile,
    Category::Installer,
    Category::TemporaryFile,
    Category::Duplicate,
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub path: String,
    pub category: Category,
    pub reason: String,
    pub size_bytes: u64,
    pub modified_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySummary {
    pub category: Category,
    pub count: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub total_files: usize,
    pub total_bytes: u64,
    pub reclaimable_bytes: u64,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisReport {
    pub summary: AnalysisSummary,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub large_file_min_bytes: u64,
    pub old_file_max_age_days: i64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            large_file_min_bytes: 100 * 1024 * 1024,
            old_file_max_age_days: 90,
        }
    }
}

pub struct AnalysisInput<'a> {
    pub files: &'a [FileEntry],
    pub config: &'a AnalysisConfig,
    pub now_ms: i64,
}

type Rule = fn(&AnalysisInput) -> Vec<Finding>;

// Add a rule by appending its function here.
const RULES: &[Rule] = &[
    large_files,
    old_files,
    installers,
    temporary_files,
    duplicates,
];

pub fn analyze(input: &AnalysisInput) -> Vec<Finding> {
    RULES.iter().copied().flat_map(|rule| rule(input)).collect()
}

pub fn report(input: &AnalysisInput) -> AnalysisReport {
    let findings = analyze(input);
    let summary = summarize(input, &findings);
    AnalysisReport { summary, findings }
}

fn summarize(input: &AnalysisInput, findings: &[Finding]) -> AnalysisSummary {
    let total_files = input.files.len();
    let total_bytes = input.files.iter().map(|f| f.size_bytes).sum();

    let categories = ALL_CATEGORIES
        .iter()
        .map(|&category| {
            let matching = findings.iter().filter(|f| f.category == category);
            let (count, bytes) = matching.fold((0, 0), |(n, b), f| (n + 1, b + f.size_bytes));
            CategorySummary {
                category,
                count,
                bytes,
            }
        })
        .collect();

    AnalysisSummary {
        total_files,
        total_bytes,
        reclaimable_bytes: reclaimable(input, findings),
        categories,
    }
}

// Each flagged file counts once, and one copy per duplicate group is kept.
fn reclaimable(input: &AnalysisInput, findings: &[Finding]) -> u64 {
    let mut flagged: HashMap<&str, u64> = HashMap::new();
    for f in findings {
        flagged.insert(&f.path, f.size_bytes);
    }
    let gross: u64 = flagged.values().sum();

    let mut counts_by_size: HashMap<u64, usize> = HashMap::new();
    for f in input.files.iter().filter(|f| f.size_bytes > 0) {
        *counts_by_size.entry(f.size_bytes).or_default() += 1;
    }
    let kept_duplicate_copies: u64 = counts_by_size
        .iter()
        .filter(|(_, n)| **n >= 2)
        .map(|(size, _)| *size)
        .sum();

    gross.saturating_sub(kept_duplicate_copies)
}

const MS_PER_DAY: i64 = 86_400_000;
const INSTALLER_EXTS: &[&str] = &["dmg", "exe", "msi", "pkg", "deb", "rpm", "appimage", "apk"];
const TEMP_EXTS: &[&str] = &["tmp", "temp", "crdownload", "part", "partial", "download"];

fn large_files(input: &AnalysisInput) -> Vec<Finding> {
    input
        .files
        .iter()
        .filter(|f| f.size_bytes >= input.config.large_file_min_bytes)
        .map(|f| {
            finding(
                f,
                Category::LargeFile,
                format!("Large file taking up {}", human_size(f.size_bytes)),
            )
        })
        .collect()
}

fn old_files(input: &AnalysisInput) -> Vec<Finding> {
    let cutoff = input.now_ms - input.config.old_file_max_age_days * MS_PER_DAY;
    input
        .files
        .iter()
        .filter_map(|f| {
            let modified = f.modified_ms?;
            if modified >= cutoff {
                return None;
            }
            let days = (input.now_ms - modified) / MS_PER_DAY;
            Some(finding(
                f,
                Category::OldFile,
                format!("Not touched in {days} days"),
            ))
        })
        .collect()
}

fn installers(input: &AnalysisInput) -> Vec<Finding> {
    flag_by_extension(
        input,
        INSTALLER_EXTS,
        Category::Installer,
        "Installer you've likely already run",
    )
}

fn temporary_files(input: &AnalysisInput) -> Vec<Finding> {
    flag_by_extension(
        input,
        TEMP_EXTS,
        Category::TemporaryFile,
        "Temporary or partial download",
    )
}

fn flag_by_extension(
    input: &AnalysisInput,
    exts: &[&str],
    category: Category,
    reason: &str,
) -> Vec<Finding> {
    input
        .files
        .iter()
        .filter(|f| f.extension.as_deref().is_some_and(|e| exts.contains(&e)))
        .map(|f| finding(f, category, reason.to_string()))
        .collect()
}

// Size matching is a metadata-only heuristic: no false negatives (identical
// content implies identical size) but possible false positives. Content hashing
// during scanning would make this exact.
fn duplicates(input: &AnalysisInput) -> Vec<Finding> {
    let mut by_size: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
    for file in input.files.iter().filter(|f| f.size_bytes > 0) {
        by_size.entry(file.size_bytes).or_default().push(file);
    }

    by_size
        .into_values()
        .filter(|group| group.len() >= 2)
        .flat_map(|group| {
            let count = group.len();
            group.into_iter().map(move |f| {
                finding(
                    f,
                    Category::Duplicate,
                    format!("Possible duplicate: {count} files share this exact size"),
                )
            })
        })
        .collect()
}

fn finding(file: &FileEntry, category: Category, reason: String) -> Finding {
    Finding {
        path: file.path.clone(),
        category,
        reason,
        size_bytes: file.size_bytes,
        modified_ms: file.modified_ms,
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, ext: Option<&str>, size: u64, modified_ms: Option<i64>) -> FileEntry {
        FileEntry {
            name: path.rsplit('/').next().unwrap().to_string(),
            extension: ext.map(str::to_string),
            path: path.to_string(),
            size_bytes: size,
            created_ms: None,
            modified_ms,
            mime_type: "application/octet-stream".to_string(),
            is_hidden: false,
        }
    }

    const NOW: i64 = 1_000 * MS_PER_DAY;

    fn config() -> AnalysisConfig {
        AnalysisConfig {
            large_file_min_bytes: 1024,
            old_file_max_age_days: 30,
        }
    }

    fn run(files: &[FileEntry]) -> Vec<Finding> {
        analyze(&AnalysisInput {
            files,
            config: &config(),
            now_ms: NOW,
        })
    }

    fn categories(findings: &[Finding], path: &str) -> Vec<Category> {
        findings
            .iter()
            .filter(|f| f.path == path)
            .map(|f| f.category)
            .collect()
    }

    #[test]
    fn flags_large_files_over_threshold() {
        let findings = run(&[
            file("/dl/big", None, 2048, Some(NOW)),
            file("/dl/small", None, 10, Some(NOW)),
        ]);
        assert_eq!(categories(&findings, "/dl/big"), vec![Category::LargeFile]);
        assert!(categories(&findings, "/dl/small").is_empty());
    }

    #[test]
    fn flags_old_files_past_age() {
        let old = NOW - 60 * MS_PER_DAY;
        let findings = run(&[
            file("/dl/old", None, 1, Some(old)),
            file("/dl/new", None, 2, Some(NOW)),
        ]);
        assert_eq!(categories(&findings, "/dl/old"), vec![Category::OldFile]);
        assert!(categories(&findings, "/dl/new").is_empty());
    }

    #[test]
    fn flags_installers_and_temp_files_by_extension() {
        let findings = run(&[
            file("/dl/app.dmg", Some("dmg"), 1, Some(NOW)),
            file("/dl/x.part", Some("part"), 2, Some(NOW)),
        ]);
        assert_eq!(
            categories(&findings, "/dl/app.dmg"),
            vec![Category::Installer]
        );
        assert_eq!(
            categories(&findings, "/dl/x.part"),
            vec![Category::TemporaryFile]
        );
    }

    #[test]
    fn flags_same_size_files_as_duplicates() {
        let findings = run(&[
            file("/dl/a", None, 500, Some(NOW)),
            file("/dl/b", None, 500, Some(NOW)),
            file("/dl/c", None, 7, Some(NOW)),
        ]);
        let dupes: Vec<&str> = findings
            .iter()
            .filter(|f| f.category == Category::Duplicate)
            .map(|f| f.path.as_str())
            .collect();
        assert_eq!(dupes.len(), 2);
        assert!(dupes.contains(&"/dl/a") && dupes.contains(&"/dl/b"));
    }

    #[test]
    fn ignores_zero_byte_files_for_duplicates() {
        let findings = run(&[
            file("/dl/a", None, 0, Some(NOW)),
            file("/dl/b", None, 0, Some(NOW)),
        ]);
        assert!(!findings.iter().any(|f| f.category == Category::Duplicate));
    }

    #[test]
    fn summary_totals_and_reclaimable_keep_one_duplicate() {
        let files = [
            file("/dl/a", None, 100, Some(NOW)),
            file("/dl/b", None, 100, Some(NOW)),
            file("/dl/c", None, 2000, Some(NOW)),
        ];
        let summary = report(&AnalysisInput {
            files: &files,
            config: &config(),
            now_ms: NOW,
        })
        .summary;

        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.total_bytes, 2200);
        assert_eq!(summary.reclaimable_bytes, 2100);

        let dup = summary
            .categories
            .iter()
            .find(|c| c.category == Category::Duplicate)
            .unwrap();
        assert_eq!((dup.count, dup.bytes), (2, 200));
        let large = summary
            .categories
            .iter()
            .find(|c| c.category == Category::LargeFile)
            .unwrap();
        assert_eq!((large.count, large.bytes), (1, 2000));
    }
}
