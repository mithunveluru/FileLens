// These types mirror the Rust structs sent over IPC; keep both sides in sync.

export interface AppInfo {
  name: string;
  version: string;
}

export interface FileEntry {
  name: string;
  extension: string | null;
  path: string;
  sizeBytes: number;
  createdMs: number | null;
  modifiedMs: number | null;
  mimeType: string;
  isHidden: boolean;
}

export interface ScanOutcome {
  files: FileEntry[];
  errorCount: number;
  cancelled: boolean;
  limitExceeded: boolean;
}

export type FindingCategory = "largeFile" | "oldFile" | "installer" | "temporaryFile";

export type VerificationStatus = "verified" | "possibleDuplicate" | "similarMetadata";

export interface DuplicateCandidate {
  path: string;
  sizeBytes: number;
  modifiedMs: number | null;
}

export interface DuplicateGroup {
  hash: string;
  status: VerificationStatus;
  sizeBytes: number;
  copies: number;
  reclaimableBytes: number;
  files: DuplicateCandidate[];
}

export interface DuplicateReport {
  groups: DuplicateGroup[];
  totalGroups: number;
  redundantFiles: number;
  reclaimableBytes: number;
  filesHashed: number;
  cacheHits: number;
  errors: string[];
  cancelled: boolean;
}

export interface Finding {
  path: string;
  category: FindingCategory;
  reason: string;
  sizeBytes: number;
  modifiedMs: number | null;
}

export interface CategorySummary {
  category: FindingCategory;
  count: number;
  bytes: number;
}

export interface AnalysisSummary {
  totalFiles: number;
  totalBytes: number;
  reclaimableBytes: number;
  categories: CategorySummary[];
}

export interface AnalysisReport {
  summary: AnalysisSummary;
  findings: Finding[];
}

export type FileKind =
  | "documents"
  | "images"
  | "videos"
  | "audio"
  | "archives"
  | "installers"
  | "code"
  | "other";

export type ActionStatus = "accepted" | "skipped";

export type ConflictStrategy = "skip" | "rename" | "replace" | "keepBoth";

export interface OrganizationAction {
  source: string;
  destination: string;
  kind: FileKind;
  reason: string;
  conflict: boolean;
  strategy: ConflictStrategy;
  status: ActionStatus;
}

export interface CategoryCount {
  kind: FileKind;
  count: number;
}

export interface PlanSummary {
  total: number;
  conflicts: number;
  categories: CategoryCount[];
}

export interface OrganizationPlan {
  root: string;
  actions: OrganizationAction[];
  summary: PlanSummary;
}

export interface ExecutionResult {
  moved: number;
  skipped: number;
  failed: number;
  sessionId: number | null;
  errors: string[];
}

export interface OrganizationSessionRecord {
  id: number;
  createdMs: number;
  root: string;
  moveCount: number;
  undone: boolean;
}

export interface UndoResult {
  restored: number;
  failed: number;
  errors: string[];
}

export interface Settings {
  /** null/empty means the OS Downloads folder. */
  downloadsFolder: string | null;
  ageThresholdDays: number;
  largeFileMinMb: number;
  ignoredFolders: string[];
  ignoredExtensions: string[];
  theme: "system" | "light" | "dark";
  autoScanOnStartup: boolean;
  launchOnStartup: boolean;
  rememberLastScanLocation: boolean;
}

export interface ScanRecord {
  id: number;
  rootPath: string;
  startedMs: number;
  finishedMs: number | null;
  fileCount: number;
  errorCount: number;
  cancelled: boolean;
}
