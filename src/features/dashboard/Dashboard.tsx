import { useMemo, useState } from "react";
import ConfirmDialog from "@/components/ConfirmDialog";
import Spinner from "@/components/Spinner";
import type { AnalysisController } from "@/features/analysis/useAnalysis";
import FilePreview from "@/features/cleanup/FilePreview";
import { useCleanup } from "@/features/cleanup/useCleanup";
import FindingsControls from "@/features/dashboard/FindingsControls";
import FindingsTable from "@/features/dashboard/FindingsTable";
import { applyView, type ViewOptions } from "@/features/dashboard/findingsView";
import OverviewCards from "@/features/dashboard/OverviewCards";
import DuplicatesPanel from "@/features/duplicates/DuplicatesPanel";
import { formatBytes } from "@/shared/format/bytes";
import { basename } from "@/shared/format/path";
import type { Finding } from "@/shared/types";
import "./Dashboard.css";

const INITIAL_VIEW: ViewOptions = {
  search: "",
  category: "all",
  sort: "sizeDesc",
  page: 0,
  pageSize: 50,
};

interface DashboardProps {
  analysis: AnalysisController;
}

function Dashboard({ analysis }: DashboardProps) {
  const { status, report, error, run } = analysis;
  const cleanup = useCleanup(run);
  const [view, setView] = useState<ViewOptions>(INITIAL_VIEW);
  const [confirmTarget, setConfirmTarget] = useState<Finding | null>(null);
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [lastIgnored, setLastIgnored] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const viewResult = useMemo(
    () => (report ? applyView(report.findings, view) : null),
    [report, view],
  );

  // Changing a filter/search/sort resets to the first page; paging keeps it.
  const updateView = (patch: Partial<ViewOptions>) =>
    setView((current) => ({ ...current, page: 0, ...patch }));

  const handleIgnore = async (path: string) => {
    try {
      await cleanup.ignore(path);
      setLastIgnored(path);
    } catch (cause) {
      setActionError(String(cause));
    }
  };

  const handleUndoIgnore = async () => {
    if (!lastIgnored) return;
    try {
      await cleanup.unignore(lastIgnored);
      setLastIgnored(null);
    } catch (cause) {
      setActionError(String(cause));
    }
  };

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

  if (status === "error") {
    return (
      <section className="dashboard">
        <p className="dashboard-error" role="alert">
          {error}
        </p>
        <button type="button" onClick={run}>
          Try again
        </button>
      </section>
    );
  }

  if (!report || !viewResult) {
    return (
      <section className="dashboard">
        <p className="dashboard-loading dashboard-note">
          <Spinner /> Analyzing your Downloads…
        </p>
      </section>
    );
  }

  const { rows, total, pageCount, page } = viewResult;

  return (
    <section className="dashboard">
      <OverviewCards summary={report.summary} />

      {actionError && (
        <p className="dashboard-error" role="alert">
          {actionError}{" "}
          <button type="button" onClick={() => setActionError(null)}>
            Dismiss
          </button>
        </p>
      )}

      {lastIgnored && (
        <p className="dashboard-undo">
          Ignored {basename(lastIgnored)}.{" "}
          <button type="button" onClick={handleUndoIgnore}>
            Undo
          </button>
        </p>
      )}

      {report.findings.length === 0 ? (
        <p className="dashboard-note">Nothing to clean up — your Downloads look tidy.</p>
      ) : (
        <>
          <FindingsControls
            options={view}
            categories={report.summary.categories.filter((c) => c.count > 0)}
            onChange={updateView}
          />

          {total === 0 ? (
            <p className="dashboard-note">No files match your filters.</p>
          ) : (
            <>
              <div className="findings-table-wrap">
                <FindingsTable
                  rows={rows}
                  onInfo={setPreviewPath}
                  onReveal={cleanup.reveal}
                  onIgnore={handleIgnore}
                  onTrash={setConfirmTarget}
                />
              </div>
              {pageCount > 1 && (
                <div className="dashboard-pagination">
                  <button
                    type="button"
                    disabled={page === 0}
                    onClick={() => updateView({ page: page - 1 })}
                  >
                    Previous
                  </button>
                  <span>
                    Page {page + 1} of {pageCount} · {total} files
                  </span>
                  <button
                    type="button"
                    disabled={page >= pageCount - 1}
                    onClick={() => updateView({ page: page + 1 })}
                  >
                    Next
                  </button>
                </div>
              )}
            </>
          )}
        </>
      )}

      <button
        type="button"
        className="dashboard-refresh"
        onClick={run}
        disabled={status === "running"}
      >
        {status === "running" ? "Refreshing…" : "Refresh analysis"}
      </button>

      <DuplicatesPanel onInventoryChange={run} />

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

      {previewPath && <FilePreview path={previewPath} onClose={() => setPreviewPath(null)} />}
    </section>
  );
}

export default Dashboard;
