import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { cancelDuplicateScan, findDuplicates } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { DuplicateReport } from "@/shared/types";

const PROGRESS_EVENT = "dedup:progress";

export type DuplicatesStatus = "idle" | "running" | "done" | "error";

export interface DuplicatesController {
  status: DuplicatesStatus;
  progress: number;
  report: DuplicateReport | null;
  error: string | null;
  run: () => Promise<void>;
  cancel: () => void;
}

// On-demand, not automatic: hashing is heavier than the metadata analysis, so
// the user triggers it explicitly.
export function useDuplicates(): DuplicatesController {
  const [status, setStatus] = useState<DuplicatesStatus>("idle");
  const [progress, setProgress] = useState(0);
  const [report, setReport] = useState<DuplicateReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<number>(PROGRESS_EVENT, (event) => {
      setProgress(event.payload);
    });
    return () => {
      unlisten.then((off) => off());
    };
  }, []);

  const run = useCallback(async () => {
    setStatus("running");
    setProgress(0);
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

  const cancel = useCallback(() => {
    void cancelDuplicateScan();
  }, []);

  return { status, progress, report, error, run, cancel };
}
