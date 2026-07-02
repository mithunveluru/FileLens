import { useState } from "react";
import ConfirmDialog from "@/components/ConfirmDialog";
import Spinner from "@/components/Spinner";
import { useCleanup } from "@/features/cleanup/useCleanup";
import { formatBytes } from "@/shared/format/bytes";
import { basename } from "@/shared/format/path";
import type { DuplicateCandidate } from "@/shared/types";
import { useDuplicates } from "./useDuplicates";
import "./Duplicates.css";

function formatDate(modifiedMs: number | null): string {
  return modifiedMs === null ? "—" : new Date(modifiedMs).toLocaleDateString();
}

function DuplicatesPanel() {
  const { status, report, error, run } = useDuplicates();
  const cleanup = useCleanup(run);
  const [confirmTarget, setConfirmTarget] = useState<DuplicateCandidate | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const confirmTrash = async () => {
    if (!confirmTarget) return;
    const { path } = confirmTarget;
    setConfirmTarget(null);
    try {
      await cleanup.trash(path);
    } catch (cause) {
      setActionError(String(cause));
    }
  };

  return (
    <section className="duplicates">
      <div className="duplicates-header">
        <div>
          <h2>Verified duplicates</h2>
          <p className="duplicates-note">
            Files are compared by content hash — only exact byte-for-byte copies are shown.
          </p>
        </div>
        <button type="button" onClick={run} disabled={status === "running"}>
          {status === "running" ? "Scanning…" : report ? "Rescan" : "Find duplicates"}
        </button>
      </div>

      {status === "running" && (
        <p className="duplicates-note">
          <Spinner /> Hashing candidates…
        </p>
      )}

      {status === "error" && (
        <p className="dashboard-error" role="alert">
          {error}
        </p>
      )}

      {actionError && (
        <p className="dashboard-error" role="alert">
          {actionError}{" "}
          <button type="button" onClick={() => setActionError(null)}>
            Dismiss
          </button>
        </p>
      )}

      {report && status !== "running" && (
        <>
          {report.totalGroups === 0 ? (
            <p className="duplicates-note">No duplicate files found.</p>
          ) : (
            <>
              <p className="duplicates-summary">
                {report.totalGroups} duplicate {report.totalGroups === 1 ? "set" : "sets"} ·{" "}
                {report.redundantFiles} redundant {report.redundantFiles === 1 ? "file" : "files"} ·{" "}
                {formatBytes(report.reclaimableBytes)} reclaimable
              </p>
              {report.groups.map((group) => (
                <div key={group.hash} className="duplicate-group">
                  <div className="duplicate-group-head">
                    {group.copies} copies · {formatBytes(group.sizeBytes)} each ·{" "}
                    {formatBytes(group.reclaimableBytes)} reclaimable
                  </div>
                  <ul>
                    {group.files.map((file) => (
                      <li key={file.path}>
                        <span className="duplicate-file" title={file.path}>
                          {basename(file.path)}
                        </span>
                        <span className="duplicate-date">{formatDate(file.modifiedMs)}</span>
                        <span className="duplicate-actions">
                          <button type="button" onClick={() => cleanup.reveal(file.path)}>
                            Open
                          </button>
                          <button
                            type="button"
                            className="danger"
                            onClick={() => setConfirmTarget(file)}
                          >
                            Trash
                          </button>
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </>
          )}

          {report.errors.length > 0 && (
            <p className="duplicates-note">
              {report.errors.length} file{report.errors.length === 1 ? "" : "s"} could not be read
              and were skipped.
            </p>
          )}
        </>
      )}

      {confirmTarget && (
        <ConfirmDialog
          title="Move to Recycle Bin?"
          message={`"${basename(confirmTarget.path)}" (${formatBytes(confirmTarget.sizeBytes)}) will be moved to the Recycle Bin. You can restore it from there.`}
          confirmLabel="Move to Recycle Bin"
          danger
          onConfirm={confirmTrash}
          onCancel={() => setConfirmTarget(null)}
        />
      )}
    </section>
  );
}

export default DuplicatesPanel;
