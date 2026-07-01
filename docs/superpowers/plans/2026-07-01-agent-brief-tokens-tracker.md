# ParseKit Agent Brief + Tokens Saved Tracker — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Execute PARSEKIT_AGENT_BRIEF.md (Sections 1–7) plus Section 8 Tokens Saved Tracker — benchmark-driven positioning, UX fixes, power features, CLI/skill, and local token savings counter.

**Architecture:** Tauri 2 + Rust sidecar for parsing; token counting via `tiktoken-rs` in Rust (shared between sidecar post-parse and benchmark script); stats persisted in `token-stats.json` alongside settings; UI in Svelte 5.

**Tech Stack:** Rust (tiktoken-rs, trash crate for replace action), Svelte 5, Tauri 2, Python benchmark script (tiktoken), tauri-plugin-global-shortcut.

**Branch:** `feat/agent-brief-tokens-tracker`

## Global Constraints

- 100% local — no network for token counting or stats
- Never show negative token savings (floor at 0)
- Scanned/OCR-only files: track "pages unlocked" separately — never mix with token savings total
- Benchmark README numbers must come only from `scripts/benchmark_tokens.py` output
- Default popover behavior unchanged when HUD toggle is off
- Parse-and-Replace moves originals to Trash only on success
- All relatable token comparisons labeled as approximate
- Run `npm run check` and `npm run test` before each task commit
- Privacy stance: explicit "stored locally, no telemetry" wherever counter shown

---

### Task 1: Commit staged clean-code refactor

**Files:** Already staged `updateState.svelte.ts`, `finderActionState.svelte.ts`, `App.svelte`

- Commit with message: `refactor(ui): extract update and finder action state modules`
- Verify: `npm run check && npm run test`

---

### Task 2: Token benchmark script + fixtures

**Files:**
- Create: `scripts/benchmark_tokens.py`
- Create: `scripts/benchmark-fixtures/` (synthetic license-safe samples or minimal generated docs)
- Create: `docs/benchmark-results.md` (generated output, committed after first run)

**Requirements:**
1. Fixture set: born-digital PDF, scanned PDF, docx, pptx, xlsx (synthetic/minimal)
2. Baseline: naive extraction (pdftotext if available, else document raw text path)
3. ParseKit: invoke sidecar or read existing parse output
4. Tokenize with tiktoken `cl100k_base` and `o200k_base`
5. Output Markdown table with before/after, % reduction, why notes
6. Export per-file-type `avg_reduction_ratio` JSON at `scripts/benchmark-ratios.json` for Task 5

**Verify:** `python3 scripts/benchmark_tokens.py` exits 0 and writes table

---

### Task 3: Settings restructure — General + File Support tabs

**Files:**
- Modify: `src/components/SettingsScreen.svelte`
- Modify: `src/locales/en.json`, `zh.json`, `es.json`
- Modify: `src/index.css` (tab styles if needed)
- Modify: `src/components/DependencyPreflight.svelte` (install action buttons)

**Requirements:**
1. Two tabs: **General** (language, appearance, launch at login, gatekeeper, finder, updates) and **File Support** (OCR language, OCR threads, converter checklist with install buttons/links)
2. Remove collapsed "Advanced" — File Support is always visible when that tab selected
3. Converter rows: PDF ✅ built-in; Office/Image show install button (brew command copy or open URL)
4. Conversion failure messages should link to File Support (add `?tab=file-support` or event)

**Verify:** `npm run check && npm run test`

---

### Task 4: Token stats storage module (Rust)

**Files:**
- Create: `src-tauri/src/token_stats.rs`
- Modify: `src-tauri/src/lib.rs` (register module + commands)
- Create: `src/lib/tokenStats.ts` (frontend IPC wrapper)
- Create: `src/lib/tokenStats.test.ts`

**Schema (`token-stats.json` in app support dir):**
```json
{
  "total_files_converted": 0,
  "total_tokens_saved": 0,
  "total_pages_unlocked": 0,
  "total_documents_unlocked": 0,
  "by_file_type": { "pdf": { "files": 0, "tokens_saved": 0 } },
  "events": [{ "ts": "ISO8601", "file_type": "pdf", "tokens_saved": 340, "pages_unlocked": 0 }]
}
```

**Commands:** `get_token_stats`, `reset_token_stats`, `record_token_savings`

**Verify:** `cargo test token_stats` + vitest

---

### Task 5: Token counting in sidecar (tiktoken-rs)

**Files:**
- Modify: `src-tauri/Cargo.toml` (add tiktoken-rs)
- Create: `src-tauri/src/token_count.rs`
- Modify: `src-tauri/src/sidecar_helpers.rs`
- Modify: `src-tauri/src/bin/parsekit-sidecar.rs`
- Read ratios from: `scripts/benchmark-ratios.json` (bundled as resource or compile-time include with defaults)

**Per-file on successful parse:**
1. Count `output_tokens` from output file text (cl100k_base primary; store both encodings optional)
2. Live baseline: naive text extract when cheap (PDF text layer only); else estimated from ratios
3. `saved = max(0, baseline - output)`
4. If scanned/OCR-required (no baseline text): increment `pages_unlocked` + `documents_unlocked`, skip token savings
5. Emit `token_savings` event in sidecar JSON line for frontend

**Verify:** unit tests for formula, floor at 0, scanned file path

---

### Task 6: Wire token stats across all conversion paths

**Files:**
- Modify: `src/App.svelte`, `src/lib/sidecar.ts`, `src/lib/progress.ts`
- Modify: `src-tauri/resources/macos/open-with-parsekit.sh` (post-parse stats update via CLI or rust helper)
- Task 7 CLI will also call record — stub hook now

**Verify:** GUI parse updates stats; integration test via mock event

---

### Task 7: Token savings UI

**Files:**
- Create: `src/components/TokenSavingsBanner.svelte`
- Create: `src/components/TokenSavingsPanel.svelte`
- Modify: `src/App.svelte` (popover quiet line)
- Modify: `src/components/SettingsScreen.svelte` or `AboutScreen.svelte` (full breakdown + reset)
- Modify: `src/locales/*.json`

**Requirements:**
1. Popover: "18,400 tokens saved this month" with lifetime/month toggle in settings
2. Settings/About: lifetime, per-type, files converted, pages unlocked (separate labeled stats)
3. Privacy note: "Counted locally on your Mac. Never sent anywhere."
4. Approximate framing: "~N ChatGPT messages" with "(approximate)" label
5. Reset counter button

**Verify:** `npm run check && npm run test`

---

### Task 8: parsekit CLI wrapper

**Files:**
- Create: `src-tauri/src/bin/parsekit-cli.rs` or `scripts/parsekit-cli.sh` wrapping sidecar
- Modify: `src-tauri/Cargo.toml`, `tauri.conf.json` if needed
- Modify: `scripts/postbuild-macos.sh` (symlink to /usr/local/bin or bundle CLI)

**Interface:**
```
parsekit convert <path> [--out <path>] [--format md|txt|json]
parsekit convert <folder> --batch [--out <folder>]
```
Exit 0 + print output path on success; non-zero + stderr on failure.
Must record token stats on success.

**Verify:** `parsekit convert --help` and convert a test file

---

### Task 9: README rewrite + benchmark table + user scenarios

**Files:**
- Modify: `README.md`
- Modify: `docs/INSTALL.md` (gatekeeper command prominent)
- Create: `AGENTS.md` (pointer to skill)

**Per PARSEKIT_AGENT_BRIEF Section 2** — only use benchmark numbers from Task 2.

**Verify:** README links to benchmark script; no invented percentages

---

### Task 10: Agent skill file

**Files:**
- Create: `skills/parsekit/SKILL.md`

Per brief Section 5.2. Verify Claude Code skill frontmatter format.

---

### Task 11: Quick Action improvements

**Files:**
- Modify: `scripts/install-finder-quick-action.sh`
- Create: second workflow "Parse to Markdown with ParseKit (Replace Original)"
- Modify: `open-with-parsekit.sh` (replace flow: trash original on success)
- Add workflow icon in `assets/branding/`

**Verify:** install script creates both workflows; replace uses Trash API

---

### Task 12: Global hotkey

**Files:**
- Modify: `src-tauri/Cargo.toml` (tauri-plugin-global-shortcut)
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/components/SettingsScreen.svelte` (configurable shortcut, default ⌃⇧M)

Parse Finder selection or clipboard file path in background.

**Verify:** hotkey triggers parse without opening popover

---

### Task 13: Clipboard-to-clipboard conversion

**Files:**
- Modify: `src-tauri/src/lib.rs` (read clipboard file paths)
- Modify: `src/App.svelte` or menu action
- Settings toggle: auto-convert on copy (off by default)

**Verify:** copy PDF in Finder → hotkey → clipboard has markdown

---

### Task 14: Floating progress HUD (opt-in)

**Files:**
- Create: `src/components/ProgressHud.svelte`
- Modify: `tauri.conf.json` (second window `progress-hud`)
- Modify: `src/App.svelte`, settings toggle

280×90px, always-on-top, shows batch progress + "+N tokens saved" on completion.

**Verify:** toggle off = current behavior only

---

### Task 15: Services menu verification + docs

- Verify Automator workflow appears in System Settings → Services
- Document in README as supported flow
- Note Shortcuts/App Intents as later phase in README

---

### Task 16: Final verification

- `npm run check && npm run test`
- `cargo test`
- `python3 scripts/benchmark_tokens.py`
- Manual smoke: popover token line, settings reset, CLI convert