import { useCallback, useEffect, useState } from "react";
import { getSettings, saveSettings } from "@/shared/ipc/commands";
import type { Settings } from "@/shared/types";

export interface SettingsController {
  /** Current settings, or null until the first load resolves. */
  value: Settings | null;
  save: (next: Settings) => Promise<void>;
}

/** Loads settings on mount and persists updates. */
export function useSettings(): SettingsController {
  const [value, setValue] = useState<Settings | null>(null);

  useEffect(() => {
    getSettings()
      .then(setValue)
      .catch(() => {
        // Logged by the IPC client; the app keeps running with defaults absent.
      });
  }, []);

  const save = useCallback(async (next: Settings) => {
    await saveSettings(next);
    setValue(next);
  }, []);

  return { value, save };
}
