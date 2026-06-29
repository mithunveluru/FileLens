import { formatBytes } from "@/shared/format/bytes";
import type { AnalysisSummary } from "@/shared/types";

interface OverviewCardsProps {
  summary: AnalysisSummary;
}

function OverviewCards({ summary }: OverviewCardsProps) {
  const cards = [
    { label: "Total files", value: summary.totalFiles.toLocaleString() },
    { label: "Disk usage", value: formatBytes(summary.totalBytes) },
    { label: "Reclaimable", value: formatBytes(summary.reclaimableBytes) },
  ];

  return (
    <div className="overview-cards">
      {cards.map((card) => (
        <div key={card.label} className="overview-card">
          <span className="overview-value">{card.value}</span>
          <span className="overview-label">{card.label}</span>
        </div>
      ))}
    </div>
  );
}

export default OverviewCards;
