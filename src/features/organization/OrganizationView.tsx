import { useEffect, useState } from "react";
import ConfirmDialog from "@/components/ConfirmDialog";
import Spinner from "@/components/Spinner";
import OrganizationHistory from "@/features/organization/OrganizationHistory";
import PlanSummaryCards from "@/features/organization/PlanSummaryCards";
import PlanTable from "@/features/organization/PlanTable";
import { useOrganizationPlan } from "@/features/organization/useOrganizationPlan";
import "./Organization.css";

/**
 * Smart Organization preview. Builds a proposed plan (read-only), lets the user
 * review and adjust each move, then executes only after explicit confirmation.
 */
function OrganizationView() {
  const {
    status,
    plan,
    result,
    error,
    generate,
    execute,
    dismissResult,
    setStatus,
    setStrategy,
    setCategory,
  } = useOrganizationPlan();
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    void generate();
  }, [generate]);

  if (status === "error") {
    return (
      <section className="organization">
        <p className="org-error" role="alert">
          {error}
        </p>
        <button type="button" onClick={generate}>
          Try again
        </button>
      </section>
    );
  }

  if (!plan) {
    return (
      <section className="organization">
        <p className="org-loading">
          <Spinner /> Building your organization plan…
        </p>
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
          Rebuild plan
        </button>
      </header>

      {result && (
        <p className="org-result" role="status">
          Moved {result.moved} file{result.moved === 1 ? "" : "s"}
          {result.skipped > 0 && `, skipped ${result.skipped}`}
          {result.failed > 0 && `, ${result.failed} failed`}.{" "}
          <button type="button" onClick={dismissResult}>
            Dismiss
          </button>
        </p>
      )}

      {plan.actions.length === 0 ? (
        <p className="org-note">
          Nothing to organize — there are no loose files in your Downloads root.
        </p>
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
            <button
              type="button"
              className="org-primary"
              disabled={executing || accepted === 0}
              onClick={() => setConfirming(true)}
            >
              {executing ? (
                <>
                  <Spinner label="Organizing" /> Organizing…
                </>
              ) : (
                `Execute plan (${accepted})`
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
