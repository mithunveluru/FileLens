import { useCallback } from "react";
import { ignorePath, revealFile, trashFile, unignorePath } from "@/shared/ipc/commands";

export interface CleanupActions {
  trash: (path: string) => Promise<void>;
  ignore: (path: string) => Promise<void>;
  unignore: (path: string) => Promise<void>;
  reveal: (path: string) => void;
}

// Each action refreshes analysis after changing the inventory.
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
