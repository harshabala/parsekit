# ParseKit — Master Report: Emil Kowalski Skills Suite

**Date:** 2026-07-16  
**Project:** `<local-path>`  
**Stack:** Tauri 2 + Svelte 5 menu-bar app (document → AI-ready Markdown)  
**Skills run (6, via isolated subagents):**

| # | Skill | Role |
|---|--------|------|
| 1 | `animation-vocabulary` | Name current motion effects |
| 2 | `apple-design` | Fluid interface / HIG-inspired audit |
| 3 | `emil-design-eng` | Design-engineering craft (UI + feedback) |
| 4 | `find-animation-opportunities` | Gaps + what not to animate |
| 5 | `improve-animations` | Self-contained improvement plans |
| 6 | `review-animations` | Brutal motion review vs 10 standards |

**Verdict consensus:** Motion system is **already disciplined** (tokens, 180/120ms, keyboard instant convert, fade-only config collapse, stagger budgets, reduced-motion hooks). Highest remaining work is **easing quality**, **dropping expensive blur**, **keyboard Escape = instant**, **deps recheck snappiness**, and **press/hover craft** — not new decoration.

---

## Executive summary

### What’s already strong (do not regress)

- Central motion API: `src/lib/motion.ts` + CSS tokens (`--timing-fast` 120ms, `--timing-normal` 180ms)
- Asymmetric enter/exit durations (180 enter / 120 exit)
- Keyboard convert path: `parseChromeInstant` → button chrome duration 0
- Background/hotkey batches: `lightMotion` / lighter section fly
- Config collapse: fade-only (no height thrash) after prior audit
- Progress bar uses `transform: scaleX`, not width
- Stagger capped ~300ms (tested in `motion.test.ts`)
- Dual reduced-motion: CSS global + JS `prefersReduced → duration 0`
- Settings panel `transform-origin: top right` (partially unused — see below)

### Cross-skill consensus priorities (fix these first)

| Rank | Finding | Skills that flagged it | Severity |
|:----:|---------|------------------------|----------|
| 1 | Exit easing is **ease-in** (`easingAccelerate`) — should be ease-out | improve, review, emil | **HIGH** |
| 2 | Soft Material ease curves — use punchier ease-out | emil, improve, review | **HIGH** |
| 3 | Panel `filter: blur` on every panel swap (GPU cost) | improve, review, apple | **HIGH** |
| 4 | Escape still animates panel out — keyboard path | review | **HIGH** |
| 5 | Deps list `slide` (height) + 180ms recheck wait | apple, emil, improve, review | **HIGH/MED** |
| 6 | Segment/back/link controls kill press scale | apple, emil | **HIGH** |
| 7 | Disabled **Convert Files** has no “why” | emil | **HIGH** |
| 8 | Dialogs: no focus trap / restore | emil | **HIGH** |
| 9 | Hover not gated with `@media (hover: hover)` | emil, review | **MED** |
| 10 | Reduced-motion is nuclear (zero everything) | apple, emil, review | **MED** |
| 11 | Progress title “Parsing → Complete” snaps | find-opportunities | **MED** |
| 12 | Settings origin set but no scale-from-gear | apple, improve | **MED** |

---

## 1. animation-vocabulary

### Glossary map (current product)

| Effect in code | Preferred term |
|----------------|----------------|
| `panelBlurFly*` (Y + opacity + blur) | **Enter/Exit** = **Slide in** + **Fade** + **Blur** |
| Svelte `fly` (banners, sections, rows) | **Slide in** + **Fade** |
| Config collapse fade | **Crossfade** (not Accordion) |
| Row/deps delays | **Stagger** + **Orchestration** |
| `deps-badge-pop` | **Pop in** (mild) |
| Button `:active` scale | **Press / Tap feedback** |
| Spinner rotate | **Rotate** + **Loop** |
| Onboarding breathe/pulse | **Idle animation** / **Pulse** |
| Progress `scaleX` | **Scale** (fill) |
| Instant keyboard path | **Reduced / purposeful** (frequency rule) |

### Team language hygiene

| Prefer | Avoid |
|--------|--------|
| Slide in + fade | “fly”, “whoosh” |
| Crossfade | “soft switch” |
| Stagger | “cascade” (unless describing the effect) |
| Accordion/collapse | Only for true height motion (deps list) — config is **fade** |
| Press feedback | “click squash” |

### Future prompts (plain language → term)

1. Settings grows from gear → **Origin-aware animation**  
2. Files cascade in → **Stagger** of **Slide in** (already have)  
3. Springy overshoot → **Pop in** / **Bounce**  
4. Digits don’t jump → **Tabular numbers** (already partly)  
5. Loading shimmer → **Skeleton / Shimmer**  
6. Invalid hotkey → **Shake / Wiggle**  

---

## 2. apple-design

### High
1. **Asymmetric enter/exit paths** — enter from +8px, exit continues −4px (not reverse path).  
2. **No springs / velocity** — all fixed cubic-bezier; fine for utility, but not “fluid.”  
3. **View stack not interruptible** — mid-flight Esc can’t re-target from live transform.  
4. **Deps recheck waits 180ms for exit anim** — input gated on cosmetics.

### Medium
5. Press feedback stripped on segments/back/links.  
6. Drop zone is boolean drag-over, not continuous 1:1 tracking.  
7. Glass is nearly opaque — blur rarely reads as material.  
8. Deps badge overshoot without gesture velocity.  
9. Infinite onboarding loops (gated under reduced-motion).  
10. Settings `transform-origin` unused without scale.  
11. Section-label tracking (0.12em) large/inconsistent.

### Low
12. Linear progress fill; hard header borders; nuclear reduced-motion; press 120ms slightly soft.

### Already aligned
Press scale on primary buttons; PRM plumbing; stagger budget; keyboard instant chrome; system body type; haptics on meaningful actions.

---

## 3. emil-design-eng

### HIGH
| ID | Finding | Fix direction |
|----|---------|---------------|
| H1 | Soft Material easings | `cubic-bezier(0.23, 1, 0.32, 1)` ease-out; sync CSS + JS |
| H2 | Disabled Convert is silent | Caption under CTA when `!canRunParse` |
| H3 | Dialogs lack focus trap/restore | Autofocus Back; trap Tab; restore gear |
| H4 | Segment hover is a no-op | Real hover wash; allow press scale |
| H5 | Back/link-cards kill press | Shared press token 0.97–0.98 |

### MEDIUM
| ID | Finding |
|----|---------|
| M1 | No `@media (hover: hover) and (pointer: fine)` |
| M2 | `select:focus` not `:focus-visible` |
| M3 | Empty history is one muted line — need next action |
| M4 | Reduced-motion zeros *all* transitions |
| M5 | Deps recheck 180ms delay + height slide |
| M6 | About linked from two places (General + File Support) |
| M7 | Onboarding system blue vs cream brand |
| M8 | Progress stats + summary redundant when complete |
| M9 | Token banner under-affords click |
| M10 | Dim icon-btn opacity elsewhere |
| M11 | Scanning drop zone text-only busy state |

### LOW
Brand “Copied!” exclamation; update banner hard-coded colors; thin 2px progress track; dual path+Change picker; slider thumb feedback.

### Don’t regress
Keyboard instant; collapse fade; primary press scale; 180/120 asymmetric; stagger budget; short icon crossfades; settings origin CSS.

---

## 4. find-animation-opportunities

### Prioritized gaps (add carefully)

| # | Sev | Location | Finding | Fix |
|---|-----|----------|---------|-----|
| 1 | Med | ProgressList title | “Parsing” → “Complete” snaps | `{#key}` + 100/80ms icon fade |
| 2 | Med | ProgressList error line | Error text expands with no fade | `hintFade` on error block only |
| 3 | Med | Onboarding steps | `in:fade` without `out:fade` | Add exit 120ms |
| 4 | Low | Onboarding checklist | 1→✓ snaps | Color/bg transition + one-shot check pop |
| 5 | Low | ProgressHud title | Same as #1 | Mirror title crossfade |

### Do **not** animate

| Location | Why |
|----------|-----|
| Keyboard convert chrome | Frequency / Emil rule — already correct |
| Live progress counts / “now parsing” name | 100s/day during batch |
| Segmented controls / format toggles | High frequency |
| Drag-over beyond current CSS | Already enough |

### Missed opportunities (optional)
Token stats empty→data fade; onboarding install error fade; drop-zone busy opacity transition; no extra delight on Open Output (already has section fly).

**Verdict:** Highest leverage is completion-state copy and error-row content — not ornament.

---

## 5. improve-animations

### Top plans (self-contained, ranked)

| # | Plan | Files | Leverage |
|---|------|-------|----------|
| **1** | Strengthen ease-out tokens globally | `index.css`, `motion.ts`, onboarding fallbacks | Max |
| **2** | Exit helpers use ease-out, not accelerate | `motion.ts` all `*Out` | High |
| **3** | Remove panel `filter: blur` (opacity+Y only) | `motion.ts`, App panels | High perf |
| **4** | Deps: fade-not-slide + remove 180ms recheck wait | `DependencyPreflight.svelte` | Med |
| **5** | Press: transform 160ms decelerate | `index.css` buttons | Med |
| **6** | Settings enter: scale 0.96→1 from top-right | `motion.ts` + settings branch | Med |
| **7** | Onboarding steps use tokenized section fly | `OnboardingScreen.svelte` | Low–med |
| **8** | Status badge crossfade stacked (no width jump) | `ProgressList` + CSS grid stack | Low–med |

**Execution order:** 1 → 2 → 3 → (4 ∥ 5) → 6 → 7–8  

**Out of scope:** Re-adding height collapse; undoing keyboard instant; Framer/springs required.

---

## 6. review-animations

### Part 1 — Findings (craft bar)

| Sev | Std | Issue | Remedial |
|-----|-----|-------|----------|
| High | 2 | Escape animates full panel out | **Delete** motion on keyboard dismiss |
| High | 3 | Exit ease-in (`easingAccelerate`) | **Fix easing** → ease-out |
| High | 7 | Animated `filter: blur` on view swaps | **Delete/reduce** blur |
| High | 7 | Deps `slide` height | **Delete** → fade |
| Med | 3 | Weak Material curves | Stronger ease-out tokens |
| Med | 1–2 | Recheck waits for animation | **Delete** delay |
| Med | 9 | Symmetric press/release 120ms | Asymmetric press/release |
| Med | 8 | Hover not pointer-gated | Gate hover motion |
| Med | 8 | PRM nuclear kill | Gentler: opacity only |
| Low | 7 | Complete pulse via `filter: brightness` | Opacity or skip |
| Low | 1–4 | Onboarding infinite loops + long delayed enter | One-shot / shorter |

### Part 2 — Verdict

**REQUEST CHANGES**

Ship when:
1. Escape (keyboard view change) is instant  
2. Exits use ease-out (not accelerate)  
3. Panel transitions are opacity ± tiny Y (no blur)  
4. Deps list is fade-not-slide, no intentional recheck delay  

---

## Unified backlog (recommended)

### P0 — Feel + performance (next PR)

1. **Ease tokens:** decelerate → `cubic-bezier(0.23, 1, 0.32, 1)`; exits use decelerate not accelerate (`motion.ts` + CSS).  
2. **Remove `filter: blur` from `panelBlurCss`** (rename helpers eventually).  
3. **Escape → instant panel swap** (flag `keyboardChromeInstant` like convert).  
4. **DependencyPreflight:** remove 180ms sleep; list fade-only (no `slide`).  
5. **Press/hover:** restore press scale on segments/back/links; real segment hover; optional hover media query.

### P1 — Craft & a11y

6. Disabled Convert: one-line reason.  
7. Settings/History/About: focus trap + restore.  
8. Progress title + HUD title crossfade; error line fade.  
9. Reduced-motion: keep short opacity, kill translate/blur only.  
10. Settings enter: subtle scale from top-right (if blur already gone).

### P2 — Polish / brand

11. Onboarding: tokenized step fly, exit fade, kill or one-shot loops; brand color cohesion.  
12. Empty history structured empty state.  
13. About single entry point.  
14. Status badge grid-stack crossfade.  
15. Brand copy: no `"Copied!"`.

### Explicit non-goals

- Animate keyboard convert chrome  
- Animate live token/progress counters mid-batch  
- Segment sliding “pill” indicators  
- Springs/Framer for every surface (optional later for drag only)  
- Marketing-page-length motion  

---

## Skill-by-skill one-liners

| Skill | Bottom line |
|--------|-------------|
| **animation-vocabulary** | Team should say slide/fade/stagger/pop — not “fly.” Config is crossfade, not collapse. |
| **apple-design** | Good press + PRM; missing interruptible stack, real glass, continuous drag feel, symmetric spatial paths. |
| **emil-design-eng** | Motion OK; UI craft: disabled CTA reason, press on all controls, dialog focus, snappier deps. |
| **find-animation-opportunities** | Few true gaps; complete-state labels and onboarding exits; protect high-freq paths. |
| **improve-animations** | Concrete plan: easings → exit ease-out → kill blur → deps → press → settings origin. |
| **review-animations** | **REQUEST CHANGES** — Escape, exit ease-in, blur, deps slide. |

---

## Suggested next step

Implement **P0 items 1–5** in one focused PR (extends prior motion-audit work). Say if you want that executed next with subagent-driven development.

---

## Artifacts

- This report (Desktop): `<local-path>`  
- Prior motion plan: `docs/superpowers/plans/2026-07-16-motion-audit-findings.md`  
- Prior motion fixes already on `master` (keyboard convert, config fade, settings about, light batch motion, etc.)

*End of master report.*
