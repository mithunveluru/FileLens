//! Smart Organization: classify Downloads files into category folders via a
//! planning-first flow. The planner only *proposes* an [`OrganizationPlan`];
//! nothing touches the filesystem until the user approves and execution runs.
//!
//! Module boundaries mirror the rest of the backend: `classifier`, `planner`,
//! and `conflict` are pure (no Tauri, no filesystem), so they are deterministic
//! and unit-tested; `commands` is the thin orchestrator.

pub mod classifier;
pub mod commands;
pub mod conflict;
pub mod execution;
pub mod planner;

use serde::{Deserialize, Serialize};

/// The category a file is organized into. Serialized as camelCase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileKind {
    Documents,
    Images,
    Videos,
    Audio,
    Archives,
    Installers,
    Code,
    Other,
}

pub(crate) const ALL_KINDS: [FileKind; 8] = [
    FileKind::Documents,
    FileKind::Images,
    FileKind::Videos,
    FileKind::Audio,
    FileKind::Archives,
    FileKind::Installers,
    FileKind::Code,
    FileKind::Other,
];

impl FileKind {
    /// The destination subfolder name inside Downloads.
    pub fn folder_name(self) -> &'static str {
        match self {
            FileKind::Documents => "Documents",
            FileKind::Images => "Images",
            FileKind::Videos => "Videos",
            FileKind::Audio => "Audio",
            FileKind::Archives => "Archives",
            FileKind::Installers => "Installers",
            FileKind::Code => "Code",
            FileKind::Other => "Other",
        }
    }
}

/// Whether the user wants a proposed action carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionStatus {
    Accepted,
    Skipped,
}

/// How to resolve a destination that already exists at execution time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictStrategy {
    Skip,
    Rename,
    Replace,
    KeepBoth,
}

/// A single proposed move. The frontend may edit `destination`, `strategy`, and
/// `status` before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAction {
    pub source: String,
    pub destination: String,
    pub kind: FileKind,
    pub reason: String,
    /// True if the destination already exists on disk at plan time.
    pub conflict: bool,
    pub strategy: ConflictStrategy,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    pub kind: FileKind,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    pub total: usize,
    pub conflicts: usize,
    pub categories: Vec<CategoryCount>,
}

/// The proposed organization, returned for preview before any changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPlan {
    pub root: String,
    pub actions: Vec<OrganizationAction>,
    pub summary: PlanSummary,
}

/// The result of executing a plan, returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub moved: usize,
    pub skipped: usize,
    pub failed: usize,
    /// The history session id, present when at least one file was moved.
    pub session_id: Option<i64>,
    pub errors: Vec<String>,
}

/// The result of undoing a session, returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub restored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}
