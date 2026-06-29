import { useEffect, useState } from "react";
import "@/components/modal.css";
import { useModalA11y } from "@/components/useModalA11y";
import { formatBytes } from "@/shared/format/bytes";
import { getFileInfo } from "@/shared/ipc/commands";
import type { FileEntry } from "@/shared/types";

interface FilePreviewProps {
  path: string;
  onClose: () => void;
}

function formatDate(ms: number | null): string {
  return ms === null ? "—" : new Date(ms).toLocaleString();
}

/** Read-only modal showing full metadata for a single file. */
function FilePreview({ path, onClose }: FilePreviewProps) {
  const [info, setInfo] = useState<FileEntry | null>(null);
  const ref = useModalA11y(onClose);

  useEffect(() => {
    getFileInfo(path)
      .then(setInfo)
      .catch(() => {
        // Logged by the IPC client; the modal shows what it can.
      });
  }, [path]);

  const rows: Array<[string, string]> = info
    ? [
        ["Name", info.name],
        ["Type", info.mimeType],
        ["Size", formatBytes(info.sizeBytes)],
        ["Created", formatDate(info.createdMs)],
        ["Modified", formatDate(info.modifiedMs)],
        ["Path", info.path],
      ]
    : [];

  return (
    <div className="modal-backdrop">
      <div
        className="modal"
        ref={ref}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label="File information"
      >
        <h2>File information</h2>
        {info === null ? (
          <p>Loading…</p>
        ) : (
          <dl className="file-preview">
            {rows.map(([label, value]) => (
              <div key={label}>
                <dt>{label}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>
        )}
        <div className="modal-actions">
          <button type="button" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

export default FilePreview;
