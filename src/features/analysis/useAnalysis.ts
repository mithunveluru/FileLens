import { useCallback, useEffect, useState } from "react";
import { analyzeDownloads } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { AnalysisReport } from "@/shared/types";

export type AnalysisStatus = "idle" | "running" | "done" | "error";

export interface AnalysisController {
  status: AnalysisStatus;
  report: AnalysisReport | null;
  error: string | null;
  run: () => Promise<void>;
}

export function useAnalysis(): AnalysisController {
  const [status, setStatus] = useState<AnalysisStatus>("idle");
  const [report, setReport] = useState<AnalysisReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    setStatus("running");
    setError(null);
    try {
      setReport(await analyzeDownloads());
      setStatus("done");
    } catch (cause) {
      logger.error(`Analysis failed: ${String(cause)}`);
      setError(String(cause));
      setStatus("error");
    }
  }, []);

  useEffect(() => {
    void run();
  }, [run]);

  return { status, report, error, run };
}
