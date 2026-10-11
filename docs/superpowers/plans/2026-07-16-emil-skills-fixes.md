# Emil Skills Master Report — Implementation Plan

**Branch:** `fix/emil-skills-master-fixes`  
**Source:** Desktop master report + `docs/superpowers/plans/2026-07-16-emil-skills-master-report.md`

## Global Constraints

- Keep UI durations ≤ 300ms; enter 180 / exit 120 where applicable
- Keyboard-initiated UI (⌘R, Escape) = **no** motion (duration 0)
- Animate transform/opacity only (no height slide for popover chrome)
- Prefer ease-out for enter **and** exit (no ease-in on UI exits)
- No new motion libraries
- Sync CSS tokens with `src/lib/motion.ts` cubics
- Commit with `[skip release]` unless cutting a release
- `npm run check` + `npm test` must pass

## Tasks

### T1 — Easing foundation
- `--easing-decelerate: cubic-bezier(0.23, 1, 0.32, 1)`
- Optional stronger standard: `cubic-bezier(0.77, 0, 0.175, 1)` for morphs only
- All exit helpers use `easingDecelerate` not `easingAccelerate`
- Update onboarding hardcoded curves

### T2 — Kill panel blur
- `panelBlurCss` → transform + opacity only
- Keep Y 8 / −4

### T3 — Escape / keyboard view chrome instant
- Flag when Escape closes settings/history/about
- Instant out transitions for those paths

### T4 — DependencyPreflight
- Remove 180ms recheck delay
- List container: fade not slide
- Install block already fades

### T5 — Press + hover craft
- Segment hover wash + allow press scale
- Back/link-card/collapse press scale
- `@media (hover: hover) and (pointer: fine)` for hover motion

### T6 — Disabled Convert reason
- Caption under CTA when `!canRunParse`

### T7 — Progress / HUD title crossfade + error line fade

### T8 — Reduced-motion gentler
- Under PRM: allow short opacity, zero transform/filter

### T9 — Settings enter scale from origin (0.96→1) top-right, no blur

### T10 — Focus trap on Settings/History/About (autofocus Back, restore trigger)

### T11 — Polish
- Empty history next action
- About single path (optional - keep if low risk)
- "Copied!" → "Copied"
- Onboarding step out:fade
- Status badge grid stack if quick
