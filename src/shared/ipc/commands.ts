import { call } from "@/shared/ipc/client";
import type {
  AnalysisReport,
  AppInfo,
  DuplicateReport,
  ExecutionResult,
  FileEntry,
  OrganizationPlan,
  OrganizationSessionRecord,
  ScanOutcome,
  ScanRecord,
  Settings,
  UndoResult,
} from "@/shared/types";

// One wrapper per Rust #[tauri::command]; the whole IPC surface lives here.

export const getAppInfo = (): Promise<AppInfo> => call<AppInfo>("app_info");

export const scanDownloads = (): Promise<ScanOutcome> => call<ScanOutcome>("scan_downloads");

export const cancelScan = (): Promise<void> => call<void>("cancel_scan");

export const getScanHistory = (limit = 10): Promise<ScanRecord[]> =>
  call<ScanRecord[]>("scan_history", { limit });

export const analyzeDownloads = (): Promise<AnalysisReport> =>
  call<AnalysisReport>("analyze_downloads");

export const findDuplicates = (): Promise<DuplicateReport> =>
  call<DuplicateReport>("find_duplicates");

export const cancelDuplicateScan = (): Promise<void> => call<void>("cancel_duplicate_scan");

export const trashFile = (path: string): Promise<void> => call<void>("trash_file", { path });

export const revealFile = (path: string): Promise<void> => call<void>("reveal_file", { path });

export const ignorePath = (path: string): Promise<void> => call<void>("ignore_path", { path });

export const unignorePath = (path: string): Promise<void> => call<void>("unignore_path", { path });

export const getFileInfo = (path: string): Promise<FileEntry | null> =>
  call<FileEntry | null>("file_info", { path });

export const getSettings = (): Promise<Settings> => call<Settings>("get_settings");

export const saveSettings = (settings: Settings): Promise<Settings> =>
  call<Settings>("save_settings", { settings });

export const generateOrganizationPlan = (): Promise<OrganizationPlan> =>
  call<OrganizationPlan>("generate_organization_plan");

export const executeOrganizationPlan = (plan: OrganizationPlan): Promise<ExecutionResult> =>
  call<ExecutionResult>("execute_organization_plan", { plan });

export const organizationHistory = (limit = 20): Promise<OrganizationSessionRecord[]> =>
  call<OrganizationSessionRecord[]>("organization_history", { limit });

export const undoOrganization = (sessionId: number): Promise<UndoResult> =>
  call<UndoResult>("undo_organization", { sessionId });
