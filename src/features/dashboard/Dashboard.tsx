import { RefreshCw, SearchX, Sparkles } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
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

  const viewResult = useMemo(
    () => (report ? applyView(report.findings, view) : null),
    [report, view],
  );

  // Changing a filter/search/sort resets to the first page; paging keeps it.
  const updateView = (patch: Partial<ViewOptions>) =>
    setView((current) => ({ ...current, page: 0, ...patch }));

  // Undo rides along in the toast, so it is attached to the action that
  // caused it rather than parked in a banner above the table.
  const handleIgnore = async (path: string) => {
    try {
      await cleanup.ignore(path);
      toast.success(`Ignored ${basename(path)}`, {
        action: {
          label: "Undo",
          onClick: () => {
            cleanup.unignore(path).catch((cause) => toast.error(String(cause)));
          },
        },
      });
    } catch (cause) {
      toast.error(String(cause));
    }
  };

  const confirmTrash = async () => {
    if (!confirmTarget) return;
    const { path, sizeBytes } = confirmTarget;
    setConfirmTarget(null);
    try {
      await cleanup.trash(path);
      toast.success(`Moved ${basename(path)} to the Recycle Bin`, {
        description: `${formatBytes(sizeBytes)} reclaimed — restore it from the Recycle Bin if needed.`,
      });
    } catch (cause) {
      toast.error(String(cause));
    }
  };

  if (status === "error") {
    return (
      <section className="dashboard">
        <p className="banner banner-danger" role="alert">
          <span>{error}</span>
          <button type="button" className="btn-sm" onClick={run}>
            Try again
          </button>
        </p>
      </section>
    );
  }

  if (!report || !viewResult) {
    // Skeleton mirrors the loaded layout so nothing jumps when data lands.
    return (
      <section className="dashboard dashboard-skeleton" aria-busy="true" aria-label="Loading">
        <div className="overview-cards">
          <div className="skeleton" />
          <div className="skeleton" />
          <div className="skeleton" />
        </div>
        <div className="dashboard-skeleton-rows">
          {[80, 95, 70, 90, 60].map((width) => (
            <div key={width} className="skeleton" style={{ width: `${width}%` }} />
          ))}
        </div>
      </section>
    );
  }

  const { rows, total, pageCount, page } = viewResult;

  return (
    <section className="dashboard">
      <div className="dashboard-head">
        <div>
          <h2>Cleanup findings</h2>
          <p className="dashboard-sub">Files worth a second look, largest payoff first.</p>
        </div>
        <button type="button" onClick={run} disabled={status === "running"}>
          {status === "running" ? <Spinner /> : <RefreshCw />}
          {status === "running" ? "Refreshing…" : "Refresh analysis"}
        </button>
      </div>

      <OverviewCards summary={report.summary} />

      {report.findings.length === 0 ? (
        <div className="empty">
          <Sparkles />
          <p className="empty-title">Nothing to clean up</p>
          <p>Your Downloads folder is already tidy.</p>
        </div>
      ) : (
        <>
          <FindingsControls
            options={view}
            categories={report.summary.categories.filter((c) => c.count > 0)}
            onChange={updateView}
          />

          {total === 0 ? (
            <div className="empty">
              <SearchX />
              <p className="empty-title">No matches</p>
              <p>No files match your current search and filters.</p>
            </div>
          ) : (
            <>
              <div className="table-wrap">
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
