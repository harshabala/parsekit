# Agent log

## 2026-11-18

Branch `agent/north-star`, created from `master` at `dbc99be`. Not pushed.

Workstream 0 is not done. Tracked files have no ` 2.md` copies. The local
gitignored copies that differ only restore personal paths, so they were not
kept. `docs/parsekit-screenshots` is behind `master` (`dc38b40` already has
the gallery). It was not merged.

Workstream 3. The README no longer says "~4 chars/token". Published figures
are `cl100k_base`, an 800-token approximate ChatGPT message, and 0.0%
reduction on every fixture. `node scripts/reproduce_published_numbers.mjs`
prints them. Settings copy no longer says tokens were avoided. Score 6 → 7.

Commit: `f5a4e637dc9887c79d4bf114bb6ebdbb5e495039`. Not pushed.

Checks: `npm test` — 58 passed, 0 failed. The sidecar benchmark was not
re-run. No new measurement was invented.

Needs the owner: confirm the North Star, supply or approve the real document
set, and agree targets. Do not merge to `master` from this branch.

## 2026-11-23

Press and loading states on the existing buttons. A press scales to 0.97
or 0.98 and darkens, then returns. `prefers-reduced-motion` disables the
scale and keeps a still darken. Busy controls (update install, drop zone
scan, dependency recheck, Finder install, update check, onboarding install,
token reset) keep their text and get a still inset border. No spinner
library. Parsing and token claims were not changed.

Commit: `ea5e668840dfd8dc302afb045a24bfb320e940bd`. Not pushed.

Checks: `npm test` — 60 passed, 0 failed. Not verified in the running app.

Impeccable panel fixes. Copy Last File is next to Open Output after a batch. The drop zone is before output settings; the panel size is unchanged. The drop hint and onboarding agree that Word needs an extra converter. The OCR hint and a failed file's error are in the text, not a tooltip. Reduced motion keeps the converting text and does not use a spinner as the only cue. The job is named Convert, not Parse, in the button, status, and clipboard label. The 3px active-row stripe is gone. `BatchScoreboard 2.svelte` is unimported and untested; it was left in place.

Commit: `2c974277533b669209b808610e66a2fd86078aa5`. Not pushed.

Checks: `npm test` — 60 passed, 0 failed. Not verified in the running app. No new measurement was invented.
