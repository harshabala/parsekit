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
export const MOTION_ENTER_MS = 180;
export const MOTION_EXIT_MS = 120;

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