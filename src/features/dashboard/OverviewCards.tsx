import { Files, HardDrive, Sparkles } from "lucide-react";
import { formatBytes } from "@/shared/format/bytes";
import type { AnalysisSummary } from "@/shared/types";

interface OverviewCardsProps {
  summary: AnalysisSummary;
}

function OverviewCards({ summary }: OverviewCardsProps) {
  const cards = [
    { label: "Total files", value: summary.totalFiles.toLocaleString(), Icon: Files },
    { label: "Disk usage", value: formatBytes(summary.totalBytes), Icon: HardDrive },
  ];

  return (
    <div className="overview-cards">
      {cards.map(({ label, value, Icon }) => (
        <div key={label} className="stat">
          <span className="stat-value">{value}</span>
          <span className="stat-label">
            <Icon />
            {label}
          </span>
        </div>
      ))}

      {/* Reclaimable is the number the whole app exists to produce, so it gets
          the gradient treatment rather than sitting level with the others. */}
      <div className="stat stat-hero">
        <span className="stat-value">{formatBytes(summary.reclaimableBytes)}</span>
        <span className="stat-label">
          <Sparkles />
          Reclaimable
        </span>
      </div>
    </div>
  );
}

export default OverviewCards;
