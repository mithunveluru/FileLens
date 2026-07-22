import { AlertTriangle, Check } from "lucide-react";
import Tip from "@/components/Tip";
import { KIND_FOLDERS, KIND_ORDER } from "@/features/organization/destination";
import { basename } from "@/shared/format/path";
import type { ConflictStrategy, FileKind, OrganizationAction } from "@/shared/types";
import { KIND_FACETS } from "@/shared/ui/tones";

interface PlanTableProps {
  actions: OrganizationAction[];
  onCategory: (index: number, kind: FileKind) => void;
  onStrategy: (index: number, strategy: ConflictStrategy) => void;
  onStatus: (index: number, status: OrganizationAction["status"]) => void;
}

const STRATEGY_LABELS: Record<ConflictStrategy, string> = {
  keepBoth: "Keep both",
  rename: "Rename",
  replace: "Replace",
  skip: "Skip",
};

function PlanTable({ actions, onCategory, onStrategy, onStatus }: PlanTableProps) {
  return (
    <div className="table-wrap">
      <table className="data-table org-table">
        <thead>
          <tr>
            <th>Run</th>
            <th>File</th>
            <th>Move to</th>
            <th>Why</th>
            <th>Conflict</th>
          </tr>
        </thead>
        <tbody>
          {actions.map((action, index) => {
            const accepted = action.status === "accepted";
            const name = basename(action.source);
            const facet = KIND_FACETS[action.kind];
            return (
              <tr key={action.source} className={accepted ? "" : "org-row-skipped"}>
                <td>
                  <input
                    type="checkbox"
                    aria-label={`Move ${name}`}
                    checked={accepted}
                    onChange={(e) =>
                      onStatus(index, e.currentTarget.checked ? "accepted" : "skipped")
                    }
                  />
                </td>
                <td className="org-name">
                  <Tip content={action.source}>
                    <span>{name}</span>
                  </Tip>
                </td>
                <td>
                  {/* The swatch colours the destination the same way the summary
                      chips do, so the folder is recognisable before reading it. */}
                  <span className="org-destination" data-tone={facet.tone}>
                    <facet.Icon />
                    <select
                      aria-label={`Destination for ${name}`}
                      value={action.kind}
                      onChange={(e) => onCategory(index, e.currentTarget.value as FileKind)}
                    >
                      {KIND_ORDER.map((kind) => (
                        <option key={kind} value={kind}>
                          {KIND_FOLDERS[kind]}
                        </option>
                      ))}
                    </select>
                  </span>
                </td>
                <td className="org-reason">{action.reason}</td>
                <td>
                  {action.conflict ? (
                    <span className="org-conflict" data-tone="amber">
                      <AlertTriangle />
                      <select
                        aria-label={`Conflict strategy for ${name}`}
                        value={action.strategy}
                        onChange={(e) =>
                          onStrategy(index, e.currentTarget.value as ConflictStrategy)
                        }
                      >
                        {(Object.keys(STRATEGY_LABELS) as ConflictStrategy[]).map((strategy) => (
                          <option key={strategy} value={strategy}>
                            {STRATEGY_LABELS[strategy]}
                          </option>
                        ))}
                      </select>
                    </span>
                  ) : (
                    <span className="org-ok" data-tone="emerald">
                      <Check />
                      Clear
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export default PlanTable;
