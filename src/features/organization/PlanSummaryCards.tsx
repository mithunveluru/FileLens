import { KIND_FOLDERS } from "@/features/organization/destination";
import type { PlanSummary } from "@/shared/types";

interface PlanSummaryCardsProps {
  summary: PlanSummary;
  /** Number of actions currently set to run. */
  accepted: number;
}

/** Headline stats for the proposed plan. */
function PlanSummaryCards({ summary, accepted }: PlanSummaryCardsProps) {
  return (
    <div className="org-summary">
      <div className="org-cards">
        <div className="org-card">
          <span className="org-value">{accepted}</span>
          <span className="org-label">Files to move</span>
        </div>
        <div className="org-card">
          <span className="org-value">{summary.categories.length}</span>
          <span className="org-label">Destination folders</span>
        </div>
        <div className="org-card">
          <span className="org-value">{summary.conflicts}</span>
          <span className="org-label">Conflicts</span>
        </div>
      </div>
      <div className="org-chips">
        {summary.categories.map((category) => (
          <span key={category.kind} className="org-chip">
            {KIND_FOLDERS[category.kind]} · {category.count}
          </span>
        ))}
      </div>
    </div>
  );
}

export default PlanSummaryCards;
