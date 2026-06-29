import { call } from "@/shared/ipc/client";
import type {
  AnalysisReport,
  AppInfo,
  ExecutionResult,
  FileEntry,
  OrganizationPlan,
  OrganizationSessionRecord,
  ScanOutcome,
  ScanRecord,
  Settings,
  UndoResult,
} from "@/shared/types";

/**
 * The single registry of backend commands available to the frontend.
 *
 * One function per Rust `#[tauri::command]`. As features land, their commands
 * are added here so the full IPC surface is discoverable in one file.
 */

/** Returns the application name and version from the Rust backend. */
export const getAppInfo = (): Promise<AppInfo> => call<AppInfo>("app_info");

/** Scans the Downloads folder and resolves with the full file inventory. */
export const scanDownloads = (): Promise<ScanOutcome> => call<ScanOutcome>("scan_downloads");

/** Requests cancellation of the in-progress scan, if any. */
export const cancelScan = (): Promise<void> => call<void>("cancel_scan");

/** Returns the most recent persisted scans, newest first. */
export const getScanHistory = (limit = 10): Promise<ScanRecord[]> =>
  call<ScanRecord[]>("scan_history", { limit });

/** Runs the analysis engine over the stored inventory and returns the report. */
export const analyzeDownloads = (): Promise<AnalysisReport> =>
  call<AnalysisReport>("analyze_downloads");

/** Moves a Downloads file to the OS Recycle Bin (recoverable from there). */
export const trashFile = (path: string): Promise<void> => call<void>("trash_file", { path });

/** Opens the system file manager with the file selected. */
export const revealFile = (path: string): Promise<void> => call<void>("reveal_file", { path });

/** Excludes a path from future analysis. */
export const ignorePath = (path: string): Promise<void> => call<void>("ignore_path", { path });

/** Restores a previously ignored path. */
export const unignorePath = (path: string): Promise<void> => call<void>("unignore_path", { path });

/** Returns full metadata for a file, for the preview panel. */
export const getFileInfo = (path: string): Promise<FileEntry | null> =>
  call<FileEntry | null>("file_info", { path });

/** Returns the current user settings (defaults if none saved). */
export const getSettings = (): Promise<Settings> => call<Settings>("get_settings");

/** Persists the given user settings. */
export const saveSettings = (settings: Settings): Promise<void> =>
  call<void>("save_settings", { settings });

/** Builds a proposed organization plan (read-only — no filesystem changes). */
export const generateOrganizationPlan = (): Promise<OrganizationPlan> =>
  call<OrganizationPlan>("generate_organization_plan");

/** Executes an approved organization plan, moving files and recording history. */
export const executeOrganizationPlan = (plan: OrganizationPlan): Promise<ExecutionResult> =>
  call<ExecutionResult>("execute_organization_plan", { plan });

/** Returns recent organization sessions, newest first. */
export const organizationHistory = (limit = 20): Promise<OrganizationSessionRecord[]> =>
  call<OrganizationSessionRecord[]>("organization_history", { limit });

/** Reverses a recorded organization session. */
export const undoOrganization = (sessionId: number): Promise<UndoResult> =>
  call<UndoResult>("undo_organization", { sessionId });
