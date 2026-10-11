/**
 * Local rolling job outcomes for success-rate display (PK-3).
 * Counts only — no paths, names, or document content.
 */
import { getSetting, setSetting } from "./store";

export const JOB_OUTCOMES_KEY = "jobOutcomesV1";
export const JOB_OUTCOMES_CAP = 20;

export interface JobOutcome {
  /** ISO timestamp */
  ts: string;
  success: number;
  failed: number;
  tokensSaved: number;
}

export interface JobOutcomesState {
  jobs: JobOutcome[];
}

export function emptyJobOutcomes(): JobOutcomesState {
  return { jobs: [] };
}

export function recordJobOutcome(
  state: JobOutcomesState,
  outcome: Omit<JobOutcome, "ts"> & { ts?: string },
  cap = JOB_OUTCOMES_CAP,
): JobOutcomesState {
  const job: JobOutcome = {
    ts: outcome.ts ?? new Date().toISOString(),
    success: Math.max(0, Math.floor(outcome.success)),
    failed: Math.max(0, Math.floor(outcome.failed)),
    tokensSaved: Math.max(0, Math.floor(outcome.tokensSaved)),
  };
  const jobs = [job, ...state.jobs].slice(0, cap);
  return { jobs };
}

export function successRate(state: JobOutcomesState): {
  success: number;
  failed: number;
  total: number;
  rate: number | null;
  label: string;
} {
  let success = 0;
  let failed = 0;
  for (const j of state.jobs) {
    success += j.success;
    failed += j.failed;
  }
  const total = success + failed;
  if (total === 0) {
    return { success: 0, failed: 0, total: 0, rate: null, label: "—" };
  }
  const rate = success / total;
  return {
    success,
    failed,
    total,
    rate,
    label: `${success}/${total}`,
  };
}

export async function loadJobOutcomes(): Promise<JobOutcomesState> {
  const raw = await getSetting<JobOutcomesState | null>(JOB_OUTCOMES_KEY, null);
  if (!raw || !Array.isArray(raw.jobs)) return emptyJobOutcomes();
  return { jobs: raw.jobs.slice(0, JOB_OUTCOMES_CAP) };
}

export async function persistJobOutcome(
  outcome: Omit<JobOutcome, "ts"> & { ts?: string },
): Promise<JobOutcomesState> {
  const prev = await loadJobOutcomes();
  const next = recordJobOutcome(prev, outcome);
  await setSetting(JOB_OUTCOMES_KEY, next);
  return next;
}
