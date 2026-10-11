import { describe, expect, it } from "vitest";
import {
  emptyJobOutcomes,
  recordJobOutcome,
  successRate,
  JOB_OUTCOMES_CAP,
} from "./jobOutcomes";

describe("jobOutcomes", () => {
  it("starts empty", () => {
    const s = successRate(emptyJobOutcomes());
    expect(s.rate).toBeNull();
    expect(s.label).toBe("—");
  });

  it("accumulates success and fail", () => {
    let state = emptyJobOutcomes();
    state = recordJobOutcome(state, { success: 3, failed: 1, tokensSaved: 100 });
    state = recordJobOutcome(state, { success: 0, failed: 2, tokensSaved: 0 });
    const s = successRate(state);
    expect(s.success).toBe(3);
    expect(s.failed).toBe(3);
    expect(s.total).toBe(6);
    expect(s.label).toBe("3/6");
    expect(s.rate).toBeCloseTo(0.5);
  });

  it("caps history", () => {
    let state = emptyJobOutcomes();
    for (let i = 0; i < JOB_OUTCOMES_CAP + 5; i++) {
      state = recordJobOutcome(state, {
        success: 1,
        failed: 0,
        tokensSaved: i,
      });
    }
    expect(state.jobs.length).toBe(JOB_OUTCOMES_CAP);
  });
});
