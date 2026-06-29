import { useCallback, useState } from "react";
import { recomputeDestination } from "@/features/organization/destination";
import { executeOrganizationPlan, generateOrganizationPlan } from "@/shared/ipc/commands";
import { logger } from "@/shared/logging/logger";
import type {
  ExecutionResult,
  FileKind,
  OrganizationAction,
  OrganizationPlan,
} from "@/shared/types";

export type PlanStatus = "idle" | "loading" | "ready" | "executing" | "error";

export interface OrganizationPlanController {
  status: PlanStatus;
  plan: OrganizationPlan | null;
  result: ExecutionResult | null;
  error: string | null;
  generate: () => Promise<void>;
  execute: (plan: OrganizationPlan) => Promise<void>;
  dismissResult: () => void;
  setStatus: (index: number, status: OrganizationAction["status"]) => void;
  setStrategy: (index: number, strategy: OrganizationAction["strategy"]) => void;
  setCategory: (index: number, kind: FileKind) => void;
}

// The plan is loaded read-only; the user's edits are applied client-side until execution.
export function useOrganizationPlan(): OrganizationPlanController {
  const [status, setStatus] = useState<PlanStatus>("idle");
  const [plan, setPlan] = useState<OrganizationPlan | null>(null);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const generate = useCallback(async () => {
    setStatus("loading");
    setError(null);
    try {
      setPlan(await generateOrganizationPlan());
      setStatus("ready");
    } catch (cause) {
      logger.error(`Failed to generate organization plan: ${String(cause)}`);
      setError(String(cause));
      setStatus("error");
    }
  }, []);

  const patchAction = useCallback((index: number, patch: Partial<OrganizationAction>) => {
    setPlan((current) =>
      current
        ? {
            ...current,
            actions: current.actions.map((action, i) =>
              i === index ? { ...action, ...patch } : action,
            ),
          }
        : current,
    );
  }, []);

  const execute = useCallback(
    async (planToRun: OrganizationPlan) => {
      setStatus("executing");
      setError(null);
      try {
        setResult(await executeOrganizationPlan(planToRun));
        // Rebuild from the new on-disk state so the preview reflects what moved.
        await generate();
      } catch (cause) {
        logger.error(`Failed to execute organization plan: ${String(cause)}`);
        setError(String(cause));
        setStatus("error");
      }
    },
    [generate],
  );

  return {
    status,
    plan,
    result,
    error,
    generate,
    execute,
    dismissResult: () => setResult(null),
    setStatus: (index, status) => patchAction(index, { status }),
    setStrategy: (index, strategy) => patchAction(index, { strategy }),
    setCategory: (index, kind) =>
      setPlan((current) => {
        if (!current) return current;
        const action = current.actions[index];
        const destination = recomputeDestination(current.root, kind, action.source);
        return {
          ...current,
          actions: current.actions.map((a, i) => (i === index ? { ...a, kind, destination } : a)),
        };
      }),
  };
}
