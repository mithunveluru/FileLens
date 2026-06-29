import { invoke } from "@tauri-apps/api/core";
import { logger } from "@/shared/logging/logger";

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (cause) {
    logger.error(`IPC command "${command}" failed: ${String(cause)}`);
    throw cause;
  }
}
