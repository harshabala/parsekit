# Motion Audit Findings — Implementation Plan

**Branch:** `fix/motion-audit-findings`  
**Source:** design-motion-principles audit (Emil primary / Jakub secondary / Jhey selective)  
**Repo:** ParseKit (Tauri + Svelte)

## Global Constraints

- Keep durations under 300ms for tray/productivity UI (Emil)
- Preserve dual reduced-motion support (CSS tokens + JS `prefersReduced` → duration 0)
- Prefer transform/opacity over layout height animation
- Do not add new animation libraries
- Match existing patterns in `src/lib/motion.ts`
- Commit message includes `[skip release]` unless releasing
- Run `npm run check` and `npm test` before finishing

## Tasks

### Task 1 — Keyboard convert: no Run/Cancel crossfade
- ⌘/⌃R (and optionally any keyboard-initiated start) should swap Run/Cancel **instantly**
- Pointer-triggered start may keep 180ms fade
- Files: `src/App.svelte`, possibly `src/lib/motion.ts`

### Task 2 — Config collapse: no height `slide`
- Replace Svelte `slide` on config card ↔ collapsed summary
- Use fade (or grid 0fr/1fr if clean); no height thrash
- Files: `src/App.svelte`, `src/lib/motion.ts` if needed

### Task 3 — Settings ↔ About symmetric enter/exit
- Both panes get paired `in:fade` + `out:fade`
- Files: `src/App.svelte`

### Task 4 — Deps install block enter/exit
- When brew/install block appears for missing deps, fade/slide in
- Files: `src/components/DependencyPreflight.svelte`

### Task 5 — Lighter motion for background/hotkey batches
- When `isBackgroundBatch` (or equivalent), reduce progress section fly and HUD entrance
- Files: `src/App.svelte`, `src/components/ProgressList.svelte`, `src/components/ProgressHud.svelte` as needed

### Task 6 — Onboarding motion hygiene
- Ensure all looping animations gated under reduced-motion
- Replace generic `ease` with project decelerate curve where practical
- Files: `src/onboarding.css`, `src/components/OnboardingScreen.svelte` if needed

### Task 7 — Polish opportunities
- Soften deps badge pop from ~0.88 → 0.95 start scale
- Optional: `transform-origin` for settings panel enter (top-right)
- Files: `src/index.css`, `src/App.svelte` as needed

## Out of scope
- Token celebration micro-interaction (product feature, not pure motion fix)
- Tooltip instant-after-first-hover system (new interaction model)
