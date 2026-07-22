import { useCallback, useEffect, useState } from "react";
import { getSettings, saveSettings } from "@/shared/ipc/commands";
import type { Settings } from "@/shared/types";

export interface SettingsController {
  /** null until the first load resolves. */
  value: Settings | null;
  save: (next: Settings) => Promise<void>;
}

export function useSettings(): SettingsController {
  const [value, setValue] = useState<Settings | null>(null);

  useEffect(() => {
    getSettings()
      .then(setValue)
      .catch(() => {
        // Logged by the IPC client; the app keeps running with defaults absent.
      });
  }, []);

  // The backend clamps invalid thresholds, so store what it actually saved.
  const save = useCallback(async (next: Settings) => {
    setValue(await saveSettings(next));
  }, []);

  return { value, save };
}
