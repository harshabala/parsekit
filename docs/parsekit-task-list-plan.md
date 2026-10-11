# parsekit — Agent task list & implementation plan

> **Give this entire file to one agent (or one session). Work only on this product until ship criteria are met.**

| Field | Value |
|-------|--------|
| **Product** | parsekit |
| **Priority** | P0 — primary showcase |
| **GitHub** | https://github.com/harshabala/parsekit |
| **Local clone (preferred)** | `~/Documents/Projects/parsekit` |
| **If missing, clone to** | `~/Documents/Projects/parsekit` via `gh repo clone harshabala/parsekit` |
| **Default branch** | `master` |
| **Work branch** | `feat/flow-metrics-2026-07` (create from latest default if needed) |
| **Stack** | Mac menu-bar document converter (Tauri/node/Rust hybrid — verify from package.json) |
| **Task IDs** | PK-1, PK-2, PK-3, PK-4, PK-5 |
| **Fable plan status** | Fable plan FAILED (session limit) — full agent plan reconstructed below from task list |
| **Work already done** | None for this initiative. Local clone may not exist — clone if needed. |

## Agent instructions (start here)

1. **Read this whole file** before editing.
2. **cd** to the local clone path above. If the path does not exist, clone the GitHub repo there.
3. `git fetch origin && git checkout -B feat/flow-metrics-2026-07 origin/master` (or continue existing feature branch if present).
4. Implement tasks in the **recommended commit sequence** at the bottom of the plan section.
5. After each commit: run the **verify commands** listed below.
6. Create/update **`docs/TASKS.md`** with every task ID and status.
7. Complete the **README dual-section rewrite**.
8. Run design + code + security self-review on the diff; fix critical/high issues.
9. Push branch and open a PR against `master`. Do **not** force-push. Do **not** merge unless the user asked for auto-merge.
10. Do **not** touch other portfolio repos in this session.

## What “done” means for this product

All task IDs listed above are implemented **or** explicitly documented as infeasible with a narrow alternative. Metrics are local-first, activation is a named constant, scoreboard is user-visible with method + privacy copy, README is dual-section, `docs/TASKS.md` is accurate, tests/build green, PR open.


## Global constraints (apply to ALL work)

Source: `~/Desktop/portfolio-implementation-plan-2026-07-10.md` and `~/Desktop/product-flow-metrics-task-list.md`.

1. **Local-first metrics** — counters/estimates on device or extension storage only. Nothing leaves unless user explicitly exports or opts in. **No cloud analytics in v1.**
2. **One activation event** — named constant in code (e.g. `ACTIVATION_EVENT = '...'`).
3. **User-visible scoreboard** on the natural success screen (ParseKit token-savings pattern), with **method + caveat microcopy**. Honest estimates; never fake precision.
4. **Privacy line on every stats surface:** e.g. *"Stored only on this device. Never uploaded."* (wording may adapt slightly per product).
5. **Zero-AI default** where relevant — core loop works without API keys; AI is optional upgrade.
6. **README rewrite required** in every repo (see below).
7. **`docs/TASKS.md`** committed checklist; update checkboxes as you finish task IDs.
8. **Ship path:** branch `feat/flow-metrics-2026-07` → push → open PR → (if user requested auto-merge and reviews pass) squash-merge to default branch and delete feature branch.

### README overhaul (mandatory for every product)

Rewrite `README.md` with **two top sections**:

#### Start here (novice)
Plain everyday words, zero jargon:
- What it is
- What problems it solves
- Step-by-step install and first use
- Edge cases and how to still use it when you hit them

#### For technical users
- Mermaid architecture diagram (GitHub-native render)
- How it works under the hood
- Metrics / activation definition
- Dev install, test commands, storage/privacy notes

### Ship checklist per repo

- [ ] All task IDs for this product implemented or honestly deferred with reason in PR
- [ ] `docs/TASKS.md` updated (checkboxes match reality)
- [ ] README novice + technical sections done
- [ ] Build/tests pass (commands in this file)
- [ ] Design pass (hierarchy, motion, empty states, reduced-motion)
- [ ] Code pass (edge cases, no regressions)
- [ ] Security pass (secrets, XSS, storage, permissions, no accidental egress)
- [ ] Branch pushed; PR opened with body mapping commits → task IDs → metrics
- [ ] Merge only if user wants auto-merge **or** explicitly asks

### Related docs (read if needed)

| File | Why |
|------|-----|
| `~/Desktop/product-flow-metrics-task-list.md` | Original product task specs + metrics definitions |
| `~/Desktop/pm-portfolio-analysis-lenny.md` | Why these metrics matter (senior PM framing) |
| `~/Desktop/portfolio-implementation-plan-2026-07-10.md` | Multi-repo orchestration notes |
| `~/Desktop/fable-portfolio-handoff/STATUS.md` | What Fable completed (mostly plans only) |



## Work already completed

None for flow-metrics. Product may already have per-file token estimates — preserve and elevate them.

If `~/Documents/Projects/parsekit` is missing:
```bash
gh repo clone harshabala/parsekit ~/Documents/Projects/parsekit
```

## Original product task spec (100% of P0 parsekit section)

# P0 — parsekit

## Product in one line

Offline Mac menu-bar app: PDFs/Office/scans → clean Markdown so more real content fits AI context and nothing leaves the machine.

## Current gap (from analysis)

Strong ROI story in README; need the **first-run path** and **return path** to revolve around the outcome number (tokens/docs), and activation to be explicit.

---

### Task PK-1 — Make first successful convert the hero of onboarding

| Field | Detail |
|-------|--------|
| **Flow change** | First launch: skip multi-step theory. One screen: drop **one** file (or “Try sample PDF”) → convert → land on result with token estimate + “Open folder” + “Convert more.” |
| **Where** | Onboarding / first-run wizard → result panel |
| **How it helps** | Activation becomes inevitable in &lt;2 minutes. Matches Lenny: aha = first successful convert. |
| **Acceptance** | New user reaches token-savings result without opening Settings. |

### Task PK-2 — Elevate token savings as the session summary (not only per-file)

| Field | Detail |
|-------|--------|
| **Flow change** | After each job **and** in menu-bar popup: show **Today** and **All time** — files converted, estimated tokens saved, optional “≈ N more messages of context.” Keep per-file estimate on the detail row (already exist). |
| **Where** | Job complete UI, menu-bar status, optional lightweight “Stats” in settings |
| **How it helps** | Retention lever: user opens app to *see cumulative value*, not only to convert. Portfolio: clear outcome metric. |
| **Metric: Activation** | `activated = first job with status success AND tokens_estimate computed` |
| **Metric: Engagement** | `files_converted` per day/week (local counter) |
| **Metric: Outcome (user-visible)** | `tokens_saved_estimate_total` — same method as per-file, summed |
| **Metric: Retention (user-visible proxy)** | “Last convert: X days ago” + streak or “files this week” |
| **Privacy** | Counters in local app storage only. No file contents, paths, or names in aggregate stats (optional: store hashed path for dedupe — prefer **no paths**). |
| **User understanding** | Tooltip: *“Estimate = tokens in raw extract vs cleaned Markdown (≈4 chars/token). Not exact — different models tokenize differently.”* Same honesty as current section. |
| **Acceptance** | After 3 files, All-time tokens &gt; 0 and matches sum of per-file estimates within rounding. |

### Task PK-3 — Failure path that still teaches value

| Field | Detail |
|-------|--------|
| **Flow change** | On failed/partial parse: show what *did* extract + “tokens recovered so far” + one clear next action (retry OCR / different format / open logs). |
| **How it helps** | Senior PM nuance: edge cases. Reduces drop-off after first bad file. |
| **Metric** | `job_success_rate` (local, last 20 jobs) shown as “18/20 succeeded” |
| **Privacy** | Error codes only, not document text. |
| **Acceptance** | Failed job never dead-ends with empty state. |

### Task PK-4 — Optional “where did you use this?” (qualitative, local)

| Field | Detail |
|-------|--------|
| **Flow change** | Soft prompt after first convert: *Used this in…* Claude / ChatGPT / Gemini / Obsidian / Other / Skip. Store choice locally; never required. |
| **How it helps** | Portfolio + product learning without cloud analytics. |
| **Metric** | Distribution of destinations (local only; exportable later as anonymous summary if you choose). |
| **Privacy** | Explicit; skippable; no network. |
| **Acceptance** | Skip is one click; no nag after dismiss. |

### Task PK-5 — Privacy strip on every stats surface

| Field | Detail |
|-------|--------|
| **Flow change** | Fixed microcopy under stats: *“All estimates stay on your Mac. ParseKit never uploads your documents.”* |
| **How it helps** | Reinforces category advantage; reduces “is this phoning home?” anxiety. |
| **Acceptance** | Visible on result + Stats. |

**ParseKit north star (for you):** weekly files successfully converted among users who activated.  
**User-facing north star:** tokens saved (all time) + files this week.

---

## Implementation plan for agent


## Implementation plan (agent-authored from product task list — Fable never produced a plan)

### Repo orientation
- Mac menu-bar app: likely Tauri + web UI (`src/`, `src-tauri/`, packaging).
- Confirm structure with `ls` and `package.json` / `src-tauri/Cargo.toml`.
- Token estimate UI may already exist — **extend** it; do not invent a second system.

### Foundation
1. Named constant e.g. `ACTIVATION_EVENT = 'first_successful_convert_with_token_estimate'` in a small shared metrics module (TS or Rust side as fits).
2. Local store for: `filesConvertedTotal`, `tokensSavedEstimateTotal`, `lastConvertAt`, `jobSuccessLast20[]` or counts, optional destination enum counters.
3. Privacy: no file paths/names in aggregate stats; no upload of document content for metrics.

### PK-1 — First-run hero convert
- Onboarding: drop one file OR “Try sample” → convert → land on result with token estimate + Open folder + Convert more.
- Files: onboarding UI components, sample asset if needed.
- Acceptance: new user reaches savings result without opening Settings.

### PK-2 — Today + All-time token summary
- After each job and in menu-bar popup: Today / All-time files + tokens saved; optional “≈ N more messages of context.”
- Method tooltip: same honesty as existing per-file estimate (chars/token caveat).
- Privacy strip under stats.

### PK-3 — Failure path
- Partial extract + tokens recovered so far + next action; show success rate last 20 jobs.

### PK-4 — Optional destination prompt
- Soft prompt after first convert; skippable; local only.

### PK-5 — Privacy strip
- Fixed microcopy on every stats surface.

### README
- Novice: install DMG, convert one PDF, understand tokens saved.
- Technical: mermaid pipeline PDF→engine→md; offline architecture.

### Verify
```bash
# Use repo’s real scripts, e.g.:
npm test  # or pnpm / cargo test as present
# Manual: run app, convert sample, confirm counters
```

### Commit sequence
1. feat(metrics): activation constant + local counters
2. feat(onboarding): first successful convert hero (PK-1)
3. feat(stats): today/all-time token scoreboard (PK-2)
4. feat(convert): failure path + success rate (PK-3)
5. feat(onboarding): optional destination prompt (PK-4)
6. docs: privacy strips + README dual sections + docs/TASKS.md


## Agent completion checklist

- [ ] Clone/path verified; branch created
- [ ] PK-1 … PK-5
- [ ] README dual sections
- [ ] docs/TASKS.md
- [ ] Build/tests
- [ ] Security: offline-first, no doc upload for metrics
- [ ] PR opened against master
