import { useEffect, useState } from "react";
import { getScanHistory } from "@/shared/ipc/commands";
import type { ScanRecord } from "@/shared/types";

/**
 * Loads recent scans, refetching whenever `refreshToken` changes (e.g. a new
 * scan completes). Passing the latest scan result as the token is enough to
 * keep the history current without any shared state.
 */
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
