import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { cancelScan, scanDownloads } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { ScanOutcome } from "@/shared/types";

const PROGRESS_EVENT = "scan:progress";

export type ScanStatus = "idle" | "scanning" | "done" | "error";

export interface ScanController {
  status: ScanStatus;
  progress: number;
  elapsedSeconds: number;
  result: ScanOutcome | null;
  error: string | null;
  start: () => Promise<void>;
  cancel: () => void;
}

export function useScan(): ScanController {
  const [status, setStatus] = useState<ScanStatus>("idle");
  const [progress, setProgress] = useState(0);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [result, setResult] = useState<ScanOutcome | null>(null);
  const [error, setError] = useState<string | null>(null);
  const startedAt = useRef(0);

  useEffect(() => {
    const unlisten = listen<number>(PROGRESS_EVENT, (event) => {
      setProgress(event.payload);
    });
    return () => {
      unlisten.then((off) => off());
    };
  }, []);

  useEffect(() => {
    if (status !== "scanning") return;
    const id = window.setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startedAt.current) / 1000));
    }, 1000);
    return () => window.clearInterval(id);
  }, [status]);

  const start = useCallback(async () => {
    startedAt.current = Date.now();
    setStatus("scanning");
    setProgress(0);
    setElapsedSeconds(0);
    setResult(null);
    setError(null);
    try {
      const outcome = await scanDownloads();
      setResult(outcome);
      setProgress(outcome.files.length);
      setStatus("done");
    } catch (cause) {
      logger.error(`Scan failed: ${String(cause)}`);
      setError(String(cause));
      setStatus("error");
    }
  }, []);

  const cancel = useCallback(() => {
    void cancelScan();
  }, []);

  return { status, progress, elapsedSeconds, result, error, start, cancel };
}
