import { invoke } from "@tauri-apps/api/core";
import { logger } from "@/shared/logging/logger";

/**
 * Thin typed wrapper around Tauri's `invoke`.
 *
 * Centralises error logging so every command failure is reported consistently.
 * Feature code calls the named functions in `commands.ts` rather than `invoke`
 * (or this `call`) directly, keeping command names in one place.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (cause) {
    logger.error(`IPC command "${command}" failed: ${String(cause)}`);
    throw cause;
  }
}
