import { KIND_FOLDERS, KIND_ORDER } from "@/features/organization/destination";
import { basename } from "@/shared/format/path";
import type { ConflictStrategy, FileKind, OrganizationAction } from "@/shared/types";

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
    <div className="org-table-wrap">
      <table className="org-table">
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
            return (
              <tr key={action.source} className={accepted ? "" : "org-row-skipped"}>
                <td>
                  <input
                    type="checkbox"
                    aria-label={`Move ${basename(action.source)}`}
                    checked={accepted}
                    onChange={(e) =>
                      onStatus(index, e.currentTarget.checked ? "accepted" : "skipped")
                    }
                  />
                </td>
                <td title={action.source}>{basename(action.source)}</td>
                <td>
                  <select
                    aria-label={`Destination for ${basename(action.source)}`}
                    value={action.kind}
                    onChange={(e) => onCategory(index, e.currentTarget.value as FileKind)}
                  >
                    {KIND_ORDER.map((kind) => (
                      <option key={kind} value={kind}>
                        {KIND_FOLDERS[kind]}
                      </option>
                    ))}
                  </select>
                </td>
                <td>{action.reason}</td>
                <td>
                  {action.conflict ? (
                    <select
                      aria-label={`Conflict strategy for ${basename(action.source)}`}
                      value={action.strategy}
                      onChange={(e) => onStrategy(index, e.currentTarget.value as ConflictStrategy)}
                    >
                      {(Object.keys(STRATEGY_LABELS) as ConflictStrategy[]).map((strategy) => (
                        <option key={strategy} value={strategy}>
                          {STRATEGY_LABELS[strategy]}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <span className="org-ok">—</span>
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
