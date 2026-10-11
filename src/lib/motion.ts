/**
 * Panel enter/exit params for Svelte `fly` / `fade` transitions.
 * Easing curves match CSS tokens `--easing-decelerate` and `--easing-standard`.
 * UI enters AND exits use ease-out (decelerate). No ease-in on exits (Emil / review bar).
 */

function cubicBezier(x1: number, y1: number, x2: number, y2: number): (t: number) => number {
  const ax = 3 * x1 - 3 * x2 + 1;
  const bx = 3 * x2 - 6 * x1;
  const cx = 3 * x1;
  const ay = 3 * y1 - 3 * y2 + 1;
  const by = 3 * y2 - 6 * y1;
  const cy = 3 * y1;

  const sampleX = (t: number) => ((ax * t + bx) * t + cx) * t;
  const sampleY = (t: number) => ((ay * t + by) * t + cy) * t;
  const sampleDerivativeX = (t: number) => (3 * ax * t + 2 * bx) * t + cx;

  const solveCurveX = (x: number) => {
    let t2 = x;
    for (let i = 0; i < 8; i++) {
      const err = sampleX(t2) - x;
      if (Math.abs(err) < 1e-6) return t2;
      const d = sampleDerivativeX(t2);
      if (Math.abs(d) < 1e-6) break;
      t2 -= err / d;
    }
    let lo = 0;
    let hi = 1;
    t2 = x;
    while (lo < hi) {
      const err = sampleX(t2) - x;
      if (Math.abs(err) < 1e-6) return t2;
      if (x > err) lo = t2;
      else hi = t2;
      t2 = (lo + hi) / 2;
    }
    return t2;
  };

  return (t: number) => {
    if (t <= 0) return 0;
    if (t >= 1) return 1;
    return sampleY(solveCurveX(t));
  };
}

/** Strong ease-out: cubic-bezier(0.23, 1, 0.32, 1) — matches `--easing-decelerate` */
export const easingDecelerate = cubicBezier(0.23, 1, 0.32, 1);

/**
 * @deprecated Prefer easingDecelerate for UI exits. Kept for any external import.
 * Historically ease-in; now aliases decelerate so exits stay responsive.
 */
export const easingAccelerate = easingDecelerate;

export const MOTION_ENTER_Y = 8;
export const MOTION_EXIT_Y = -4;
export const MOTION_BANNER_ENTER_Y = 4;
export const MOTION_BANNER_EXIT_Y = -2;
/** Panel blur removed (GPU cost); kept at 0 for API stability. */
export const MOTION_ENTER_BLUR_PX = 0;
export const MOTION_ENTER_MS = 180;
export const MOTION_EXIT_MS = 120;
export const MOTION_HINT_MS = 120;
/** Max stagger index so delay×index + duration stays ≤ MOTION_STAGGER_BUDGET_MS */
export const MOTION_ROW_STAGGER_MAX = 5;
export const MOTION_ROW_STAGGER_DELAY_MS = 32;
export const MOTION_ROW_STAGGER_CAP = 15;
export const MOTION_ROW_ENTER_MS = 140;
export const MOTION_STAGGER_BUDGET_MS = 300;
export const MOTION_DEPS_STAGGER_DELAY_MS = 40;
export const MOTION_DEPS_ENTER_MS = 180;
/** Keep in sync with `--motion-deps-pop` in index.css */
export const MOTION_DEPS_POP_MS = 220;

function cappedStaggerDelayMs(index: number, durationMs: number): number {
  const maxIndex = Math.floor(
    (MOTION_STAGGER_BUDGET_MS - durationMs) / MOTION_DEPS_STAGGER_DELAY_MS,
  );
  return Math.min(index, Math.max(0, maxIndex)) * MOTION_DEPS_STAGGER_DELAY_MS;
}

/** Deps list row fly-in stagger (capped to MOTION_STAGGER_BUDGET_MS) */
export function depsStaggerDelayMs(index: number): number {
  return cappedStaggerDelayMs(index, MOTION_DEPS_ENTER_MS);
}

/** Deps badge pop stagger (capped to MOTION_STAGGER_BUDGET_MS) */
export function depsPopDelayMs(index: number): number {
  return cappedStaggerDelayMs(index, MOTION_DEPS_POP_MS);
}

/** Opacity + translateY only (no filter:blur — GPU-friendly). */
function panelFlyCss(t: number, u: number, y: number, scaleFrom = 1): string {
  const scale = scaleFrom === 1 ? 1 : scaleFrom + (1 - scaleFrom) * t;
  const scalePart = scaleFrom === 1 ? "" : ` scale(${scale})`;
  return `transform: translateY(${y * u}px)${scalePart}; opacity: ${t};`;
}

export type BlurFlyTransition = {
  duration: number;
  easing: (t: number) => number;
  css: (t: number, u: number) => string;
};

/** Params for panel enter (translateY + opacity). Name kept for call sites. */
export function panelBlurFlyInParams(
  prefersReduced: boolean,
  options?: { instant?: boolean; originScale?: boolean },
): BlurFlyTransition {
  const instant = options?.instant === true || prefersReduced;
  const y = instant ? 0 : MOTION_ENTER_Y;
  const duration = instant ? 0 : MOTION_ENTER_MS;
  const scaleFrom = !instant && options?.originScale ? 0.96 : 1;
  return {
    duration,
    easing: easingDecelerate,
    css: (t: number, u: number) => panelFlyCss(t, u, y, scaleFrom),
  };
}

export function panelBlurFlyOutParams(
  prefersReduced: boolean,
  options?: { instant?: boolean },
): BlurFlyTransition {
  const instant = options?.instant === true || prefersReduced;
  const y = instant ? 0 : MOTION_EXIT_Y;
  const duration = instant ? 0 : MOTION_EXIT_MS;
  return {
    duration,
    easing: easingDecelerate,
    css: (t: number, u: number) => panelFlyCss(t, u, y, 1),
  };
}

/** Svelte transition: panel enter. */
export function panelBlurFlyIn(_node: Element, params: BlurFlyTransition) {
  return params;
}

/** Svelte transition: panel exit. */
export function panelBlurFlyOut(_node: Element, params: BlurFlyTransition) {
  return params;
}

export function panelFlyIn(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_ENTER_Y,
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function panelFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

export function panelFadeIn(prefersReduced: boolean) {
  return {
    // Under reduced motion: keep a short opacity crossfade for comprehension
    duration: prefersReduced ? 100 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function panelFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 80 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

export function bannerFlyIn(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_BANNER_ENTER_Y,
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function bannerFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_BANNER_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

/** Progress section enter: y 8, 180ms ease-out */
export function sectionFlyIn(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_ENTER_Y,
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

/** Progress section exit: y -4, 120ms ease-out */
export function sectionFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

/** Run/cancel button crossfade enter */
export function buttonFadeIn(prefersReduced: boolean) {
  return panelFadeIn(prefersReduced);
}

/** Run/cancel button crossfade exit */
export function buttonFadeOut(prefersReduced: boolean) {
  return panelFadeOut(prefersReduced);
}

export const MOTION_ICON_ENTER_MS = 100;
export const MOTION_ICON_EXIT_MS = 80;

/** Status icon crossfade enter */
export function iconFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ICON_ENTER_MS,
    easing: easingDecelerate,
  };
}

/** Status icon crossfade exit */
export function iconFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ICON_EXIT_MS,
    easing: easingDecelerate,
  };
}

/** Inline hints, status lines, drop-zone ready */
export function hintFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 80 : MOTION_HINT_MS,
    easing: easingDecelerate,
  };
}

export function hintFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 60 : MOTION_HINT_MS,
    easing: easingDecelerate,
  };
}

/**
 * Config card ↔ collapsed summary.
 * Fade only (no height slide) — avoids layout thrash in the menu-bar popover.
 */
export function collapseFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 80 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function collapseFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 60 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

/** @deprecated Prefer collapseFadeIn */
export function collapseSlideIn(prefersReduced: boolean) {
  return collapseFadeIn(prefersReduced);
}

/** @deprecated Prefer collapseFadeOut */
export function collapseSlideOut(prefersReduced: boolean) {
  return collapseFadeOut(prefersReduced);
}

/**
 * Button / section fades that can be forced instant (keyboard path, background batches).
 */
export function buttonFadeInMaybeInstant(
  prefersReduced: boolean,
  instant: boolean,
) {
  return {
    duration: prefersReduced || instant ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function buttonFadeOutMaybeInstant(
  prefersReduced: boolean,
  instant: boolean,
) {
  return {
    duration: prefersReduced || instant ? 0 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

export function sectionFlyInMaybeLight(
  prefersReduced: boolean,
  light: boolean,
) {
  return {
    y: prefersReduced || light ? 0 : MOTION_ENTER_Y,
    duration: prefersReduced ? 0 : light ? MOTION_HINT_MS : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function sectionFlyOutMaybeLight(
  prefersReduced: boolean,
  light: boolean,
) {
  return {
    y: prefersReduced || light ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : light ? MOTION_HINT_MS : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}

/** Settings ↔ About sub-view */
export function subviewFadeOut(prefersReduced: boolean) {
  return panelFadeOut(prefersReduced);
}

/** File row stagger on batch start (≤15 files, capped by MOTION_STAGGER_BUDGET_MS). */
export function rowFlyIn(
  prefersReduced: boolean,
  index: number,
  stagger: boolean,
) {
  const capped = Math.min(index, MOTION_ROW_STAGGER_MAX);
  return {
    y: prefersReduced ? 0 : 6,
    duration: prefersReduced ? 0 : MOTION_ROW_ENTER_MS,
    delay: prefersReduced || !stagger ? 0 : capped * MOTION_ROW_STAGGER_DELAY_MS,
    easing: easingDecelerate,
  };
}

export function rowFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingDecelerate,
  };
}
