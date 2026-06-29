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
      <div className="scan-actions">
        <button type="button" onClick={start} disabled={scanning}>
          {scanning ? "Scanning…" : "Scan Downloads"}
        </button>
        {scanning && (
          <button type="button" className="scan-cancel" onClick={cancel}>
            Cancel
          </button>
        )}
      </div>

      {scanning && (
        <p className="scan-status" aria-live="polite">
          <Spinner /> Scanning files — {progress} found · {elapsedSeconds}s
        </p>
      )}

      {status === "error" && (
        <p className="scan-error" role="alert">
          {error}
        </p>
      )}

      {status === "done" && result && (
        <div className="scan-summary">
          <p>
            <strong>{result.files.length}</strong> files found
            {result.errorCount > 0 && ` · ${result.errorCount} skipped`}
          </p>
          {result.cancelled && <p className="scan-note">Scan was cancelled early.</p>}
        </div>
      )}

      <ScanHistory refreshToken={result} />
    </section>
  );
}

export default ScanPanel;
