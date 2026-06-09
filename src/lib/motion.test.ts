import { describe, expect, it } from "vitest";
import {
  depsPopDelayMs,
  depsStaggerDelayMs,
  MOTION_DEPS_ENTER_MS,
  MOTION_DEPS_POP_MS,
  MOTION_DEPS_STAGGER_DELAY_MS,
  MOTION_ROW_ENTER_MS,
  MOTION_ROW_STAGGER_DELAY_MS,
  MOTION_ROW_STAGGER_MAX,
  MOTION_STAGGER_BUDGET_MS,
  rowFlyIn,
} from "./motion";

describe("rowFlyIn stagger budget", () => {
  it("keeps worst-case enter within 300ms", () => {
    const worst = rowFlyIn(false, MOTION_ROW_STAGGER_MAX, true);
    const total = (worst.delay ?? 0) + worst.duration;
    expect(total).toBeLessThanOrEqual(MOTION_STAGGER_BUDGET_MS);
  });

  it("uses zero motion when reduced motion is preferred", () => {
    const params = rowFlyIn(true, 10, true);
    expect(params.duration).toBe(0);
    expect(params.delay).toBe(0);
    expect(params.y).toBe(0);
  });
});

describe("depsStaggerDelayMs", () => {
  it("keeps worst-case deps row enter within 300ms", () => {
    const total = depsStaggerDelayMs(99) + MOTION_DEPS_ENTER_MS;
    expect(total).toBeLessThanOrEqual(MOTION_STAGGER_BUDGET_MS);
  });

  it("caps delay for high indices", () => {
    const maxIndex = Math.floor(
      (MOTION_STAGGER_BUDGET_MS - MOTION_DEPS_ENTER_MS) / MOTION_DEPS_STAGGER_DELAY_MS
    );
    expect(depsStaggerDelayMs(99)).toBe(depsStaggerDelayMs(maxIndex));
  });
});

describe("row stagger constants", () => {
  it("aligns row enter duration with stagger cap", () => {
    const total =
      MOTION_ROW_STAGGER_MAX * MOTION_ROW_STAGGER_DELAY_MS + MOTION_ROW_ENTER_MS;
    expect(total).toBe(MOTION_STAGGER_BUDGET_MS);
  });
});

describe("depsPopDelayMs", () => {
  it("caps badge pop chain within 300ms", () => {
    const worstIndex = 10;
    const total = depsPopDelayMs(worstIndex) + MOTION_DEPS_POP_MS;
    expect(total).toBeLessThanOrEqual(MOTION_STAGGER_BUDGET_MS);
  });

  it("increases delay for early indices then caps", () => {
    expect(depsPopDelayMs(0)).toBe(0);
    expect(depsPopDelayMs(1)).toBe(MOTION_DEPS_STAGGER_DELAY_MS);
    expect(depsPopDelayMs(99)).toBe(depsPopDelayMs(2));
  });
});