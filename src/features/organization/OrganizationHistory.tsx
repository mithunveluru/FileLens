import { useEffect, useState } from "react";
import ConfirmDialog from "@/components/ConfirmDialog";
import { organizationHistory, undoOrganization } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type { OrganizationSessionRecord, UndoResult } from "@/shared/types";

interface OrganizationHistoryProps {
  refreshToken: unknown;
  onUndone: () => void;
}

function OrganizationHistory({ refreshToken, onUndone }: OrganizationHistoryProps) {
  const [sessions, setSessions] = useState<OrganizationSessionRecord[]>([]);
  const [reload, setReload] = useState(0);
  const [confirmId, setConfirmId] = useState<number | null>(null);
  const [outcome, setOutcome] = useState<UndoResult | null>(null);
  const [error, setError] = useState<string | null>(null);

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
    setError(null);
    setOutcome(null);
    try {
      setOutcome(await undoOrganization(sessionId));
    } catch (cause) {
      logger.error(`Undo failed: ${String(cause)}`);
      setError(String(cause));
    } finally {
      setReload((n) => n + 1);
      onUndone();
    }
  };

  return (
    <section className="org-history">
      <h3>Organization history</h3>

      {error && (
        <p className="org-error" role="alert">
          {error}{" "}
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </p>
      )}

      {outcome && (
        <p
          className={outcome.failed > 0 ? "org-error" : "org-result"}
          role={outcome.failed > 0 ? "alert" : "status"}
        >
          Restored {outcome.restored} file{outcome.restored === 1 ? "" : "s"}
          {outcome.failed > 0 && `, ${outcome.failed} could not be restored`}.
          {outcome.errors.length > 0 && ` ${outcome.errors[0]}`}{" "}
          <button type="button" onClick={() => setOutcome(null)}>
            Dismiss
          </button>
        </p>
      )}

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
