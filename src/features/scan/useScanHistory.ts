import { useEffect, useState } from "react";
import { getScanHistory } from "@/shared/ipc/commands";
import type { ScanRecord } from "@/shared/types";

export function useScanHistory(refreshToken: unknown): ScanRecord[] {
  const [scans, setScans] = useState<ScanRecord[]>([]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: refreshToken is an intentional refetch trigger, not read in the effect body.
  useEffect(() => {
    getScanHistory()
      .then(setScans)
      .catch(() => {
        // Error already logged by the IPC client; history just stays empty.
      });
  }, [refreshToken]);

  return scans;
}
