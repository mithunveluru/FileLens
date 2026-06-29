import { debug, error, info, warn } from "@tauri-apps/plugin-log";
import { env } from "@/shared/config/env";

// Forwards to tauri-plugin-log; level filtering here keeps suppressed logs off the IPC bridge.

export type LogLevel = "debug" | "info" | "warn" | "error";

const RANK: Record<LogLevel, number> = { debug: 0, info: 1, warn: 2, error: 3 };

export function shouldLog(level: LogLevel, min: LogLevel): boolean {
  return RANK[level] >= RANK[min];
}

const configured = env.logLevel as LogLevel;
const min: LogLevel = configured in RANK ? configured : "info";

const sinks: Record<LogLevel, (message: string) => Promise<void>> = {
  debug,
  info,
  warn,
  error,
};

function emit(level: LogLevel, message: string): void {
  if (!shouldLog(level, min)) return;
  // Fire-and-forget: logging must never block callers or throw on a dead sink.
  void sinks[level](message).catch(() => {});
}

export const logger = {
  debug: (message: string) => emit("debug", message),
  info: (message: string) => emit("info", message),
  warn: (message: string) => emit("warn", message),
  error: (message: string) => emit("error", message),
};
