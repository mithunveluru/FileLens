import { useEffect, useState } from "react";
import ConfirmDialog from "@/components/ConfirmDialog";
import { organizationHistory, undoOrganization } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { OrganizationSessionRecord } from "@/shared/types";

interface OrganizationHistoryProps {
  /** Changing this refetches the list (e.g. after a new execution). */
  refreshToken: unknown;
  /** Called after an undo so the caller can rebuild the plan. */
  onUndone: () => void;
}

/** Lists past organization sessions and offers to undo each one. */
function OrganizationHistory({ refreshToken, onUndone }: OrganizationHistoryProps) {
  const [sessions, setSessions] = useState<OrganizationSessionRecord[]>([]);
  const [reload, setReload] = useState(0);
  const [confirmId, setConfirmId] = useState<number | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: refreshToken is an intentional refetch trigger, not read in the effect body.
  useEffect(() => {
    organizationHistory()
      .then(setSessions)
      .catch(() => {
        // Logged by the IPC client; history just stays as-is.
      });
  }, [refreshToken, reload]);

  if (sessions.length === 0) return null;

  const undo = async (sessionId: number) => {
    setConfirmId(null);
    try {
      await undoOrganization(sessionId);
    } catch (cause) {
      logger.error(`Undo failed: ${String(cause)}`);
    } finally {
      setReload((n) => n + 1);
      onUndone();
    }
  };

  return (
    <section className="org-history">
      <h3>Organization history</h3>
      <ul>
        {sessions.map((session) => (
          <li key={session.id}>
            <span>{new Date(session.createdMs).toLocaleString()}</span>
            <span>
              {session.moveCount} file{session.moveCount === 1 ? "" : "s"}
            </span>
            {session.undone ? (
              <span className="org-history-undone">Undone</span>
            ) : (
              <button type="button" onClick={() => setConfirmId(session.id)}>
                Undo
              </button>
            )}
          </li>
        ))}
      </ul>

      {confirmId !== null && (
        <ConfirmDialog
          title="Undo this organization?"
          message="Each moved file will be returned to its original location. Files whose original spot is now occupied are left in place."
          confirmLabel="Undo"
          onConfirm={() => undo(confirmId)}
          onCancel={() => setConfirmId(null)}
        />
      )}
    </section>
  );
}

export default OrganizationHistory;
