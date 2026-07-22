import { AnimatePresence, motion } from "framer-motion";
import { EyeOff, FolderOpen, Info, Trash2 } from "lucide-react";
import Chip from "@/components/Chip";
import RowActions from "@/components/RowActions";
import Tip from "@/components/Tip";
import { CATEGORY_LABELS } from "@/features/dashboard/findingsView";
import { formatBytes } from "@/shared/format/bytes";
import { basename } from "@/shared/format/path";
import type { Finding } from "@/shared/types";
import { CATEGORY_FACETS } from "@/shared/ui/tones";

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
    <table className="data-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Category</th>
          <th>Size</th>
          <th>Modified</th>
          <th>Why</th>
          <th aria-label="Actions" />
        </tr>
      </thead>
      <tbody>
        {/* Rows animate out when trashed or ignored, so the list doesn't just blink. */}
        <AnimatePresence initial={false}>
          {rows.map((finding) => {
            const name = basename(finding.path);
            const facet = CATEGORY_FACETS[finding.category];
            return (
              <motion.tr
                key={`${finding.category}:${finding.path}`}
                layout
                exit={{ opacity: 0, x: -16 }}
                transition={{ duration: 0.16 }}
              >
                <td className="findings-name">
                  <Tip content={finding.path}>
                    <span>{name}</span>
                  </Tip>
                </td>
                <td>
                  <Chip tone={facet.tone} Icon={facet.Icon}>
                    {CATEGORY_LABELS[finding.category]}
                  </Chip>
                </td>
                <td className="findings-num">{formatBytes(finding.sizeBytes)}</td>
                <td className="findings-num">{formatDate(finding.modifiedMs)}</td>
                <td className="findings-reason">{finding.reason}</td>
                <td className="findings-actions">
                  <RowActions
                    label={name}
                    actions={[
                      {
                        label: "File information",
                        Icon: Info,
                        onSelect: () => onInfo(finding.path),
                      },
                      {
                        label: "Show in folder",
                        Icon: FolderOpen,
                        onSelect: () => onReveal(finding.path),
                      },
                      { label: "Ignore", Icon: EyeOff, onSelect: () => onIgnore(finding.path) },
                      {
                        label: "Move to Recycle Bin",
                        Icon: Trash2,
                        onSelect: () => onTrash(finding),
                        danger: true,
                      },
                    ]}
                  />
                </td>
              </motion.tr>
            );
          })}
        </AnimatePresence>
      </tbody>
    </table>
  );
}

export default FindingsTable;
