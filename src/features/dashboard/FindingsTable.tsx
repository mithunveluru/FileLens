import { CATEGORY_LABELS } from "@/features/dashboard/findingsView";
import { formatBytes } from "@/shared/format/bytes";
import { basename } from "@/shared/format/path";
import type { Finding } from "@/shared/types";

interface FindingsTableProps {
  rows: Finding[];
  onInfo: (path: string) => void;
  onReveal: (path: string) => void;
  onIgnore: (path: string) => void;
  onTrash: (finding: Finding) => void;
}

function formatDate(modifiedMs: number | null): string {
  return modifiedMs === null ? "—" : new Date(modifiedMs).toLocaleDateString();
}

function FindingsTable({ rows, onInfo, onReveal, onIgnore, onTrash }: FindingsTableProps) {
  return (
    <table className="findings-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Category</th>
          <th>Size</th>
          <th>Modified</th>
          <th>Why</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((finding) => (
          <tr key={`${finding.category}:${finding.path}`}>
            <td title={finding.path}>{basename(finding.path)}</td>
            <td>{CATEGORY_LABELS[finding.category]}</td>
            <td>{formatBytes(finding.sizeBytes)}</td>
            <td>{formatDate(finding.modifiedMs)}</td>
            <td>{finding.reason}</td>
            <td className="findings-actions">
              <button type="button" onClick={() => onInfo(finding.path)}>
                Info
              </button>
              <button type="button" onClick={() => onReveal(finding.path)}>
                Open
              </button>
              <button type="button" onClick={() => onIgnore(finding.path)}>
                Ignore
              </button>
              <button type="button" className="danger" onClick={() => onTrash(finding)}>
                Trash
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export default FindingsTable;
