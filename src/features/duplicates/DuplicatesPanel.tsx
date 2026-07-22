import { AnimatePresence, motion } from "framer-motion";
import { CheckCircle2, Copy, FolderOpen, ScanSearch, Trash2, X } from "lucide-react";
import { useCallback, useState } from "react";
import { toast } from "sonner";
import Chip from "@/components/Chip";
import ConfirmDialog from "@/components/ConfirmDialog";
import RowActions from "@/components/RowActions";
import Spinner from "@/components/Spinner";
import Tip from "@/components/Tip";
import { useCleanup } from "@/features/cleanup/useCleanup";
import { formatBytes } from "@/shared/format/bytes";
import { basename } from "@/shared/format/path";
import type { DuplicateCandidate } from "@/shared/types";
import { useDuplicates } from "./useDuplicates";
import "./Duplicates.css";

// Groups arrive sorted by reclaimable bytes, so the first page is the payoff.
const GROUP_PAGE = 50;

function formatDate(modifiedMs: number | null): string {
  return modifiedMs === null ? "—" : new Date(modifiedMs).toLocaleDateString();
}

interface DuplicatesPanelProps {
  /** Called after a trash, so the surrounding analysis refreshes too. */
  onInventoryChange: () => void;
}

function DuplicatesPanel({ onInventoryChange }: DuplicatesPanelProps) {
  const { status, progress, report, error, run, cancel } = useDuplicates();
  const [confirmTarget, setConfirmTarget] = useState<DuplicateCandidate | null>(null);
  const [visible, setVisible] = useState(GROUP_PAGE);

  // A fresh run starts back at the first page of groups.
  const rerun = useCallback(async () => {
    setVisible(GROUP_PAGE);
    await run();
  }, [run]);

  const cleanup = useCleanup(
    useCallback(() => {
      void rerun();
      onInventoryChange();
    }, [rerun, onInventoryChange]),
  );

  const confirmTrash = async () => {
    if (!confirmTarget) return;
    const { path, sizeBytes } = confirmTarget;
    setConfirmTarget(null);
    try {
      await cleanup.trash(path);
      toast.success(`Moved ${basename(path)} to the Recycle Bin`, {
        description: `${formatBytes(sizeBytes)} reclaimed.`,
      });
    } catch (cause) {
      toast.error(String(cause));
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
        <div className="duplicates-actions">
          {status === "running" && (
            <button type="button" onClick={cancel}>
              <X />
              Cancel
            </button>
          )}
          <button type="button" onClick={rerun} disabled={status === "running"}>
            {status === "running" ? <Spinner /> : <ScanSearch />}
            {status === "running" ? "Scanning…" : report ? "Rescan" : "Find duplicates"}
          </button>
        </div>
      </div>

      {status === "running" && (
        <p className="duplicates-note" aria-live="polite">
          <Spinner /> Hashing candidates — {progress} checked
        </p>
      )}

      {status === "error" && (
        <p className="banner banner-danger" role="alert">
          <span>{error}</span>
        </p>
      )}

      {report && status !== "running" && (
        <>
          {report.cancelled && (
            <p className="duplicates-note">Stopped early — these results are partial.</p>
          )}
          {report.totalGroups === 0 ? (
            <div className="empty">
              <CheckCircle2 />
              <p className="empty-title">No duplicates</p>
              <p>Every file in your Downloads folder is unique.</p>
            </div>
          ) : (
            <>
              <p className="duplicates-summary">
                {report.totalGroups} duplicate {report.totalGroups === 1 ? "set" : "sets"} ·{" "}
                {report.redundantFiles} redundant {report.redundantFiles === 1 ? "file" : "files"} ·{" "}
                <strong>{formatBytes(report.reclaimableBytes)}</strong> reclaimable
              </p>
              {report.groups.slice(0, visible).map((group) => (
                <div key={group.hash} className="duplicate-group">
                  <div className="duplicate-group-head">
                    <Chip tone="violet" Icon={Copy}>
                      {group.copies} copies
                    </Chip>
                    <span>
                      {formatBytes(group.sizeBytes)} each · {formatBytes(group.reclaimableBytes)}{" "}
                      reclaimable
                    </span>
                  </div>
                  <ul>
                    <AnimatePresence initial={false}>
                      {group.files.map((file) => (
                        <motion.li key={file.path} layout exit={{ opacity: 0, x: -16 }}>
                          <Tip content={file.path}>
                            <span className="duplicate-file">{basename(file.path)}</span>
                          </Tip>
                          <span className="duplicate-date">{formatDate(file.modifiedMs)}</span>
                          <RowActions
                            label={basename(file.path)}
                            actions={[
                              {
                                label: "Show in folder",
                                Icon: FolderOpen,
                                onSelect: () => cleanup.reveal(file.path),
                              },
                              {
                                label: "Move to Recycle Bin",
                                Icon: Trash2,
                                onSelect: () => setConfirmTarget(file),
                                danger: true,
                              },
                            ]}
                          />
                        </motion.li>
                      ))}
                    </AnimatePresence>
                  </ul>
                </div>
              ))}
              {report.groups.length > visible && (
                <button type="button" onClick={() => setVisible((n) => n + GROUP_PAGE)}>
                  Show more ({report.groups.length - visible} remaining)
                </button>
              )}
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
