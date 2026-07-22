import { AlertTriangle, FolderTree, MoveRight } from "lucide-react";
import Chip from "@/components/Chip";
import { KIND_FOLDERS } from "@/features/organization/destination";
import type { PlanSummary } from "@/shared/types";
import { KIND_FACETS } from "@/shared/ui/tones";

interface PlanSummaryCardsProps {
  summary: PlanSummary;
  accepted: number;
}

function PlanSummaryCards({ summary, accepted }: PlanSummaryCardsProps) {
  return (
    <div className="org-summary">
      <div className="org-cards">
        <div className="stat">
          <span className="stat-value">{accepted}</span>
          <span className="stat-label">
            <MoveRight />
            Files to move
          </span>
        </div>
        <div className="stat">
          <span className="stat-value">{summary.categories.length}</span>
          <span className="stat-label">
            <FolderTree />
            Destination folders
          </span>
        </div>
        {/* Conflicts only earn colour when there actually are some. */}
        <div className="stat" data-tone={summary.conflicts > 0 ? "amber" : undefined}>
          <span className="stat-value">{summary.conflicts}</span>
          <span className="stat-label">
            <AlertTriangle />
            Conflicts
          </span>
        </div>
      </div>
      <div className="org-chips">
        {summary.categories.map((category) => {
          const facet = KIND_FACETS[category.kind];
          return (
            <Chip key={category.kind} tone={facet.tone} Icon={facet.Icon}>
              {KIND_FOLDERS[category.kind]} · {category.count}
            </Chip>
          );
        })}
      </div>
    </div>
  );
}

export default PlanSummaryCards;
