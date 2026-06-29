import { useScanHistory } from "@/features/scan/useScanHistory";

interface ScanHistoryProps {
  /** Changing this refetches the list (e.g. the latest scan result). */
  refreshToken: unknown;
}

/** Shows recent persisted scans, newest first. */
function ScanHistory({ refreshToken }: ScanHistoryProps) {
  const scans = useScanHistory(refreshToken);

  if (scans.length === 0) return null;

  return (
    <section className="scan-history">
      <h2>Recent scans</h2>
      <ul>
        {scans.map((scan) => (
          <li key={scan.id}>
            <span>{new Date(scan.startedMs).toLocaleString()}</span>
            <span>
              {scan.fileCount} files
              {scan.cancelled && " (cancelled)"}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export default ScanHistory;
