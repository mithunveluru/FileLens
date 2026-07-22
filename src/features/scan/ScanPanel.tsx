import { CheckCircle2, FolderSearch, Radar, X } from "lucide-react";
import Spinner from "@/components/Spinner";
import ScanHistory from "@/features/scan/ScanHistory";
import type { ScanController } from "@/features/scan/useScan";
import "./ScanPanel.css";

interface ScanPanelProps {
  scan: ScanController;
}

function ScanPanel({ scan }: ScanPanelProps) {
  const { status, progress, elapsedSeconds, result, error, start, cancel } = scan;
  const scanning = status === "scanning";

  return (
    <section className="scan-panel">
      <div className="scan-main">
        <span className={`scan-icon${scanning ? " scan-icon-active" : ""}`} aria-hidden="true">
          {scanning ? <Radar /> : status === "done" ? <CheckCircle2 /> : <FolderSearch />}
        </span>

        <div className="scan-copy">
          <h2>Scan your Downloads</h2>
          {scanning ? (
            <p className="scan-status" aria-live="polite">
              <Spinner /> Scanning files — {progress} found · {elapsedSeconds}s
            </p>
          ) : status === "done" && result ? (
            <p className="scan-status" aria-live="polite">
              <strong>{result.files.length.toLocaleString()}</strong> files found
              {result.errorCount > 0 && ` · ${result.errorCount} skipped`}
              {result.cancelled && " · cancelled early"}
            </p>
          ) : (
            <p className="scan-status">Take an inventory before cleaning up or organizing.</p>
          )}
        </div>

        <div className="scan-actions">
          {scanning && (
            <button type="button" onClick={cancel}>
              <X />
              Cancel
            </button>
          )}
          <button type="button" className="btn-primary" onClick={start} disabled={scanning}>
            {scanning ? <Spinner /> : <Radar />}
            {scanning ? "Scanning…" : "Scan Downloads"}
          </button>
        </div>
      </div>

      {status === "error" && (
        <p className="banner banner-danger" role="alert">
          <span>{error}</span>
        </p>
      )}

      <ScanHistory refreshToken={result} />
    </section>
  );
}

export default ScanPanel;
