import { describe, expect, it } from "vitest";
import {
  buttonFadeInMaybeInstant,
  buttonFadeOutMaybeInstant,
  collapseFadeIn,
  collapseFadeOut,
  depsPopDelayMs,
  depsStaggerDelayMs,
  MOTION_DEPS_ENTER_MS,
  MOTION_DEPS_POP_MS,
  MOTION_DEPS_STAGGER_DELAY_MS,
  MOTION_ENTER_MS,
  MOTION_ENTER_Y,
  MOTION_EXIT_MS,
  MOTION_EXIT_Y,
  MOTION_HINT_MS,
  MOTION_ROW_ENTER_MS,
  MOTION_ROW_STAGGER_DELAY_MS,
  MOTION_ROW_STAGGER_MAX,
  MOTION_STAGGER_BUDGET_MS,
  panelBlurFlyInParams,
  panelBlurFlyOutParams,
  rowFlyIn,
  sectionFlyInMaybeLight,
  sectionFlyOutMaybeLight,
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

describe("keyboard / light motion helpers", () => {
  it("forces instant panel out when instant is true (Escape chrome)", () => {
    const instantOut = panelBlurFlyOutParams(false, { instant: true });
    expect(instantOut.duration).toBe(0);
    const animatedOut = panelBlurFlyOutParams(false, { instant: false });
    expect(animatedOut.duration).toBe(MOTION_EXIT_MS);
    // Instant out must not translate (sample css at u=1).
    expect(instantOut.css(1, 1)).toContain("translateY(0px)");
    expect(animatedOut.css(1, 1)).toContain(`translateY(${MOTION_EXIT_Y}px)`);

    const instantIn = panelBlurFlyInParams(false, { instant: true });
    expect(instantIn.duration).toBe(0);
    expect(panelBlurFlyInParams(false).duration).toBe(MOTION_ENTER_MS);
    expect(panelBlurFlyInParams(false).css(1, 1)).toContain(
      `translateY(${MOTION_ENTER_Y}px)`,
    );
  });

  it("forces instant button fades when instant is true", () => {
    expect(buttonFadeInMaybeInstant(false, true).duration).toBe(0);
    expect(buttonFadeOutMaybeInstant(false, true).duration).toBe(0);
    expect(buttonFadeInMaybeInstant(false, false).duration).toBe(MOTION_ENTER_MS);
    expect(buttonFadeOutMaybeInstant(false, false).duration).toBe(MOTION_EXIT_MS);
  });

  it("uses lighter section fly for background batches", () => {
    const light = sectionFlyInMaybeLight(false, true);
    const full = sectionFlyInMaybeLight(false, false);
    expect(light.y).toBe(0);
    expect(light.duration).toBe(MOTION_HINT_MS);
    expect(full.y).toBeGreaterThan(0);
    expect(full.duration).toBe(MOTION_ENTER_MS);
    expect(sectionFlyOutMaybeLight(false, true).y).toBe(0);
  });

  it("config collapse is fade-only (no layout slide params)", () => {
    const enter = collapseFadeIn(false);
    const exit = collapseFadeOut(false);
    expect(enter.duration).toBe(MOTION_ENTER_MS);
    expect(exit.duration).toBe(MOTION_EXIT_MS);
    expect(enter).not.toHaveProperty("y");
    // PRM keeps a short opacity crossfade (gentler than hard-zero)
    expect(collapseFadeIn(true).duration).toBe(80);
    expect(collapseFadeOut(true).duration).toBe(60);
  });
});