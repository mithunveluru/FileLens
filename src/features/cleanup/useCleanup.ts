import { useCallback } from "react";
import { ignorePath, revealFile, trashFile, unignorePath } from "@/shared/ipc/commands";

export interface CleanupActions {
  /** Move a file to the Recycle Bin, then refresh. Rejects on failure. */
  trash: (path: string) => Promise<void>;
  /** Ignore a path so it stops being recommended, then refresh. */
  ignore: (path: string) => Promise<void>;
  /** Restore a previously ignored path, then refresh. */
  unignore: (path: string) => Promise<void>;
  /** Reveal a file in the system file manager (fire-and-forget). */
  reveal: (path: string) => void;
}

/**
 * Wraps the cleanup IPC commands and refreshes analysis after any action that
 * changes the inventory. Keeps action wiring out of the dashboard component.
 */
export function useCleanup(onChange: () => void): CleanupActions {
  const trash = useCallback(
    async (path: string) => {
      await trashFile(path);
      onChange();
    },
    [onChange],
  );

  const ignore = useCallback(
    async (path: string) => {
      await ignorePath(path);
      onChange();
    },
    [onChange],
  );

  const unignore = useCallback(
    async (path: string) => {
      await unignorePath(path);
      onChange();
    },
    [onChange],
  );

  const reveal = useCallback((path: string) => {
    void revealFile(path);
  }, []);

  return { trash, ignore, unignore, reveal };
}
