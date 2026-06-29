//! Smart Organization: classify, plan, then execute file moves (planning-first).

pub mod classifier;
pub mod commands;
pub mod conflict;
pub mod execution;
pub mod planner;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionStatus {
    Accepted,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictStrategy {
    Skip,
    Rename,
    Replace,
    KeepBoth,
}

// The frontend may edit destination, strategy, and status before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAction {
    pub source: String,
    pub destination: String,
    pub kind: FileKind,
    pub reason: String,
    /// Destination already exists on disk at plan time.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPlan {
    pub root: String,
    pub actions: Vec<OrganizationAction>,
    pub summary: PlanSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub moved: usize,
    pub skipped: usize,
    pub failed: usize,
    /// Present when at least one file was moved.
    pub session_id: Option<i64>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub restored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}
