/**
 * Shared types used across features and the Rust <-> React boundary.
 *
 * Types that mirror a Rust struct returned over IPC live here so both sides
 * have a single, named contract. Feature-specific types belong inside their
 * feature folder, not here.
 */

/** Basic application identity, returned by the `app_info` command. */
export interface AppInfo {
  name: string;
  version: string;
}

/** Metadata for a single scanned file. Mirrors the Rust `FileEntry` struct. */
export interface FileEntry {
  name: string;
  extension: string | null;
  path: string;
  sizeBytes: number;
  /** Creation time as Unix epoch milliseconds, when the platform reports it. */
  createdMs: number | null;
  /** Last-modified time as Unix epoch milliseconds. */
  modifiedMs: number | null;
  mimeType: string;
  isHidden: boolean;
}

/** Result of a Downloads-folder scan. Mirrors the Rust `ScanOutcome` struct. */
export interface ScanOutcome {
  files: FileEntry[];
  /** Number of entries that could not be read (e.g. permission denied). */
  errorCount: number;
  cancelled: boolean;
}

/** Why a file was flagged by the analysis engine. Mirrors Rust `Category`. */
export type FindingCategory = "largeFile" | "oldFile" | "installer" | "temporaryFile" | "duplicate";

/** A single analysis recommendation. Mirrors the Rust `Finding` struct. */
export interface Finding {
  path: string;
  category: FindingCategory;
  reason: string;
  sizeBytes: number;
  modifiedMs: number | null;
}

/** Per-category totals. Mirrors the Rust `CategorySummary` struct. */
export interface CategorySummary {
  category: FindingCategory;
  count: number;
  bytes: number;
}

/** Inventory-wide totals. Mirrors the Rust `AnalysisSummary` struct. */
export interface AnalysisSummary {
  totalFiles: number;
  totalBytes: number;
  reclaimableBytes: number;
  categories: CategorySummary[];
}

/** Full analysis result. Mirrors the Rust `AnalysisReport` struct. */
export interface AnalysisReport {
  summary: AnalysisSummary;
  findings: Finding[];
}

/** The category a file is organized into. Mirrors the Rust `FileKind` enum. */
export type FileKind =
  | "documents"
  | "images"
  | "videos"
  | "audio"
  | "archives"
  | "installers"
  | "code"
  | "other";

/** Whether a proposed organization action should run. Mirrors `ActionStatus`. */
export type ActionStatus = "accepted" | "skipped";

/** Conflict-resolution choice. Mirrors the Rust `ConflictStrategy` enum. */
export type ConflictStrategy = "skip" | "rename" | "replace" | "keepBoth";

/** A single proposed move. Mirrors the Rust `OrganizationAction` struct. */
export interface OrganizationAction {
  source: string;
  destination: string;
  kind: FileKind;
  reason: string;
  conflict: boolean;
  strategy: ConflictStrategy;
  status: ActionStatus;
}

/** Count of actions in one category. Mirrors the Rust `CategoryCount` struct. */
export interface CategoryCount {
  kind: FileKind;
  count: number;
}

/** Totals for an organization plan. Mirrors the Rust `PlanSummary` struct. */
export interface PlanSummary {
  total: number;
  conflicts: number;
  categories: CategoryCount[];
}

/** A proposed organization, previewed before execution. Mirrors `OrganizationPlan`. */
export interface OrganizationPlan {
  root: string;
  actions: OrganizationAction[];
  summary: PlanSummary;
}

/** The result of executing a plan. Mirrors the Rust `ExecutionResult` struct. */
export interface ExecutionResult {
  moved: number;
  skipped: number;
  failed: number;
  sessionId: number | null;
  errors: string[];
}

/** A past organization session. Mirrors `OrganizationSessionRecord`. */
export interface OrganizationSessionRecord {
  id: number;
  createdMs: number;
  root: string;
  moveCount: number;
  undone: boolean;
}

/** The result of undoing a session. Mirrors the Rust `UndoResult` struct. */
export interface UndoResult {
  restored: number;
  failed: number;
  errors: string[];
}

/** User-configurable settings. Mirrors the Rust `Settings` struct. */
export interface Settings {
  /** Scan-root override; null/empty means the OS Downloads folder. */
  downloadsFolder: string | null;
  ageThresholdDays: number;
  largeFileMinMb: number;
  ignoredFolders: string[];
  ignoredExtensions: string[];
  theme: "system" | "light" | "dark";
  autoScanOnStartup: boolean;
  launchOnStartup: boolean;
}

/** A persisted past scan run. Mirrors the Rust `ScanRecord` struct. */
export interface ScanRecord {
  id: number;
  rootPath: string;
  startedMs: number;
  finishedMs: number | null;
  fileCount: number;
  errorCount: number;
  cancelled: boolean;
}
