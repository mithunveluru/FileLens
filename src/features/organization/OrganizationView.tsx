import { FolderCheck, PackageCheck, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import ConfirmDialog from "@/components/ConfirmDialog";
import Spinner from "@/components/Spinner";
import OrganizationHistory from "@/features/organization/OrganizationHistory";
import PlanSummaryCards from "@/features/organization/PlanSummaryCards";
import PlanTable from "@/features/organization/PlanTable";
import { useOrganizationPlan } from "@/features/organization/useOrganizationPlan";
import "./Organization.css";

function OrganizationView() {
  const { status, plan, result, error, generate, execute, setStatus, setStrategy, setCategory } =
    useOrganizationPlan();
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    void generate();
  }, [generate]);

  // The execution summary arrives as a toast rather than a banner the user has
  // to dismiss. `result` still drives the history refresh, so it is left in state.
  useEffect(() => {
    if (!result) return;
    const summary = `Moved ${result.moved} file${result.moved === 1 ? "" : "s"}`;
    const detail = [
      result.skipped > 0 && `${result.skipped} skipped`,
      result.failed > 0 && `${result.failed} failed`,
    ]
      .filter(Boolean)
      .join(" · ");
    if (result.failed > 0) {
      toast.error(summary, { description: detail });
    } else {
      toast.success(summary, { description: detail || undefined });
    }
  }, [result]);

  if (status === "error") {
    return (
      <section className="organization">
        <p className="banner banner-danger" role="alert">
          <span>{error}</span>
          <button type="button" className="btn-sm" onClick={generate}>
            Try again
          </button>
        </p>
      </section>
    );
  }

  if (!plan) {
    return (
      <section className="organization" aria-busy="true" aria-label="Loading">
        <p className="org-loading">
          <Spinner /> Building your organization plan…
        </p>
        <div className="org-cards">
          <div className="skeleton" style={{ height: "4.7rem" }} />
          <div className="skeleton" style={{ height: "4.7rem" }} />
          <div className="skeleton" style={{ height: "4.7rem" }} />
        </div>
      </section>
    );
  }

  const accepted = plan.actions.filter((action) => action.status === "accepted").length;
  const executing = status === "executing";

  return (
    <section className="organization">
      <header className="org-header">
        <div>
          <h2>Organization plan</h2>
          <p className="org-subtitle">
            Review the proposed moves. Nothing changes until you execute the plan.
          </p>
        </div>
        <button type="button" onClick={generate} disabled={executing}>
          <RefreshCw />
          Rebuild plan
        </button>
      </header>

      {plan.actions.length === 0 ? (
        <div className="empty">
          <FolderCheck />
          <p className="empty-title">Nothing to organize</p>
          <p>There are no loose files in your Downloads root.</p>
        </div>
      ) : (
        <>
          <PlanSummaryCards summary={plan.summary} accepted={accepted} />
          <PlanTable
            actions={plan.actions}
            onCategory={setCategory}
            onStrategy={setStrategy}
            onStatus={setStatus}
          />
          <div className="org-execute">
            <p className="org-execute-hint">
              {accepted === 0
                ? "Tick at least one file to run the plan."
                : `${accepted} of ${plan.actions.length} files selected.`}
            </p>
            <button
              type="button"
              className="btn-primary"
              disabled={executing || accepted === 0}
              onClick={() => setConfirming(true)}
            >
              {executing ? (
                <>
                  <Spinner label="Organizing" /> Organizing…
                </>
              ) : (
                <>
                  <PackageCheck />
                  Execute plan ({accepted})
                </>
              )}
            </button>
          </div>
        </>
      )}

      <OrganizationHistory refreshToken={result} onUndone={generate} />

      {confirming && (
        <ConfirmDialog
          title="Organize files?"
          message={`${accepted} file${accepted === 1 ? "" : "s"} will be moved into category folders inside your Downloads folder. You can undo this from the history afterwards.`}
          confirmLabel="Execute plan"
          onConfirm={() => {
            setConfirming(false);
            void execute(plan);
          }}
          onCancel={() => setConfirming(false)}
        />
      )}
    </section>
  );
}

export default OrganizationView;
