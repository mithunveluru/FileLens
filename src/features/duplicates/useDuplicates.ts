import { useCallback, useState } from "react";
import { findDuplicates } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { DuplicateReport } from "@/shared/types";

export type DuplicatesStatus = "idle" | "running" | "done" | "error";

export interface DuplicatesController {
  status: DuplicatesStatus;
  report: DuplicateReport | null;
  error: string | null;
  run: () => Promise<void>;
}

// On-demand, not automatic: hashing is heavier than the metadata analysis, so
// the user triggers it explicitly.
export function useDuplicates(): DuplicatesController {
  const [status, setStatus] = useState<DuplicatesStatus>("idle");
  const [report, setReport] = useState<DuplicateReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    setStatus("running");
    setError(null);
    try {
      setReport(await findDuplicates());
      setStatus("done");
    } catch (cause) {
      logger.error(`Duplicate detection failed: ${String(cause)}`);
      setError(String(cause));
      setStatus("error");
    }
  }, []);

  return { status, report, error, run };
}
