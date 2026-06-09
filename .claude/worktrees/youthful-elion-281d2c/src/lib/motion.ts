/**
 * Panel enter/exit params for Svelte `fly` / `fade` transitions.
 * Easing curves match CSS tokens `--easing-decelerate` and `--easing-accelerate`.
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

/** Matches `--easing-decelerate`: cubic-bezier(0, 0, 0.2, 1) */
export const easingDecelerate = cubicBezier(0, 0, 0.2, 1);

/** Matches `--easing-accelerate`: cubic-bezier(0.4, 0, 1, 1) */
export const easingAccelerate = cubicBezier(0.4, 0, 1, 1);

export const MOTION_ENTER_Y = 8;
export const MOTION_EXIT_Y = -4;
export const MOTION_BANNER_ENTER_Y = 4;
export const MOTION_BANNER_EXIT_Y = -2;
export const MOTION_ENTER_BLUR_PX = 4;
export const MOTION_ENTER_MS = 180;
export const MOTION_EXIT_MS = 120;
export const MOTION_HINT_MS = 120;
export const MOTION_ROW_STAGGER_MAX = 9;
export const MOTION_ROW_STAGGER_DELAY_MS = 40;
export const MOTION_ROW_STAGGER_CAP = 15;

function panelBlurCss(
  t: number,
  u: number,
  y: number,
  blurPx: number
): string {
  return `transform: translateY(${y * u}px); opacity: ${t}; filter: blur(${blurPx * u}px);`;
}

export type BlurFlyTransition = {
  duration: number;
  easing: (t: number) => number;
  css: (t: number, u: number) => string;
};

/** Params for `in:panelBlurFlyIn` / `out:panelBlurFlyOut` Svelte transitions. */
export function panelBlurFlyInParams(prefersReduced: boolean): BlurFlyTransition {
  const y = prefersReduced ? 0 : MOTION_ENTER_Y;
  const blur = prefersReduced ? 0 : MOTION_ENTER_BLUR_PX;
  const duration = prefersReduced ? 0 : MOTION_ENTER_MS;
  return {
    duration,
    easing: easingDecelerate,
    css: (t: number, u: number) => panelBlurCss(t, u, y, blur),
  };
}

export function panelBlurFlyOutParams(prefersReduced: boolean): BlurFlyTransition {
  const y = prefersReduced ? 0 : MOTION_EXIT_Y;
  const blur = prefersReduced ? 0 : MOTION_ENTER_BLUR_PX;
  const duration = prefersReduced ? 0 : MOTION_EXIT_MS;
  return {
    duration,
    easing: easingAccelerate,
    css: (t: number, u: number) => panelBlurCss(t, u, y, blur),
  };
}

/** Svelte transition: panel enter (translateY + opacity + blur). */
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
    easing: easingAccelerate,
  };
}

export function panelFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function panelFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingAccelerate,
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
    easing: easingAccelerate,
  };
}

/** Progress section enter: y 8, 180ms decelerate */
export function sectionFlyIn(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_ENTER_Y,
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

/** Progress section exit: y -4, 120ms accelerate */
export function sectionFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingAccelerate,
  };
}

/** Run/cancel button crossfade enter: 180ms decelerate */
export function buttonFadeIn(prefersReduced: boolean) {
  return panelFadeIn(prefersReduced);
}

/** Run/cancel button crossfade exit: 120ms accelerate */
export function buttonFadeOut(prefersReduced: boolean) {
  return panelFadeOut(prefersReduced);
}

export const MOTION_ICON_ENTER_MS = 100;
export const MOTION_ICON_EXIT_MS = 80;

/** Status icon crossfade enter: 100ms decelerate */
export function iconFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ICON_ENTER_MS,
    easing: easingDecelerate,
  };
}

/** Status icon crossfade exit: 80ms accelerate */
export function iconFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ICON_EXIT_MS,
    easing: easingAccelerate,
  };
}

/** Inline hints, status lines, drop-zone ready: 120ms */
export function hintFadeIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_HINT_MS,
    easing: easingDecelerate,
  };
}

export function hintFadeOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_HINT_MS,
    easing: easingAccelerate,
  };
}

/** Config card ↔ collapsed summary */
export function collapseSlideIn(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_ENTER_MS,
    easing: easingDecelerate,
  };
}

export function collapseSlideOut(prefersReduced: boolean) {
  return {
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingAccelerate,
  };
}

/** Settings ↔ About sub-view */
export function subviewFadeIn(prefersReduced: boolean) {
  return panelFadeIn(prefersReduced);
}

export function subviewFadeOut(prefersReduced: boolean) {
  return panelFadeOut(prefersReduced);
}

/** File row stagger on batch start (≤15 files, cap 9 delays). */
export function rowFlyIn(
  prefersReduced: boolean,
  index: number,
  stagger: boolean
) {
  const capped = Math.min(index, MOTION_ROW_STAGGER_MAX);
  return {
    y: prefersReduced ? 0 : 6,
    duration: prefersReduced ? 0 : 160,
    delay: prefersReduced || !stagger ? 0 : capped * MOTION_ROW_STAGGER_DELAY_MS,
    easing: easingDecelerate,
  };
}

export function rowFlyOut(prefersReduced: boolean) {
  return {
    y: prefersReduced ? 0 : MOTION_EXIT_Y,
    duration: prefersReduced ? 0 : MOTION_EXIT_MS,
    easing: easingAccelerate,
  };
}