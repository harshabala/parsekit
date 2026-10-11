# ParseKit: How to Increase Adoption & Reduce Friction

**Date:** 2026-07-16  
**For:** Harsha Balakrishnan  
**Product:** ParseKit — local macOS menu-bar app that turns PDFs, Office files, and images into AI-ready Markdown/text/JSON  

This note consolidates product research on:

1. Why power users (including you) still skip ParseKit and paste into ChatGPT  
2. What the codebase already ships for low-friction workflows  
3. macOS integration opportunities  
4. Competitive / adoption patterns  
5. Ranked product wedges and a 90-day plan  

---

## 1. The real competitor

ParseKit is **not** primarily competing with Marker, Docling, or LlamaParse on parse quality.

It is competing with this habit:

> **Select PDF → drop into ChatGPT / Claude / Gemini → ask.**

That path wins because:

| Reason | Detail |
|--------|--------|
| **Zero new mental model** | The file is an *attachment to a question*, not a “pipeline step” |
| **Zero install / config** | Browser or desktop chat is already open |
| **Good enough** | Short, text-heavy PDFs (roughly 1–20 pages of prose) work fine |
| **Brand gravity** | Users choose ChatGPT/Claude and inherit whatever parser is behind upload |
| **Persistence features** | Claude Projects, ChatGPT Projects, NotebookLM absorb “I need a corpus” for many people |

### Strategy in one line

> **Stop being a place users go. Become the invisible step between “I have a file” and “⌘V in the model.”**

ParseKit only gets used when:

- **Raw upload feels worse** (tokens, privacy, scans/tables, bulk, quotas), **or**
- Cleaning is **on the path the user already walks** (Finder right-click, hotkey, clipboard, Share sheet)

---

## 2. Why you (as a power user) still skip it

| Your gesture today | Friction if using ParseKit today |
|--------------------|----------------------------------|
| Drop PDF in ChatGPT | ~1 step |
| Open ParseKit → drop → run → find `.md` → copy → paste | ~4–6 steps |

Unless the file is **scanned, multi-column, table-heavy, sensitive, or bulk**, ChatGPT wins.

So either:

- **A)** Make ParseKit ≤ **2 gestures** to paste-ready text, **or**  
- **B)** Make the quality/token win **impossible to ignore on first run**  

Preferably both.

**Core insight:** Friction is not mainly “conversion quality.” It is **context switches and gestures**. ChatGPT drag-drop is ~1 gesture. ParseKit must be **0–1 gestures to paste-ready Markdown**, or **strictly better text than native upload**.

---

## 3. Top 8 friction moments (file → AI chat)

| # | Moment | What user does today | Why ParseKit is skipped |
|---|--------|----------------------|-------------------------|
| 1 | **File already in Finder** | Right-click Open/Share/Quick Action, or drag to chat | Any step that opens a separate UI loses to drag-to-chat |
| 2 | **Need text *in* the chat box** | Upload *or* copy/paste | Upload is one gesture; convert→copy→paste is 2–3 |
| 3 | **Batch / multi-file context** | Multi-select upload or zip | Power users use Projects/folders, not a converter |
| 4 | **Browser / web PDF** | Preview → screenshot or Print to PDF → upload | Tool not in Share sheet or browser path |
| 5 | **Screenshot / image of text** | CleanShot / TextSniper / Live Text → paste | OCR competitors own “select → clipboard” |
| 6 | **Copy from Word/Pages/PDF** | Select all → paste (often messy RTF) | Auto-convert not trusted/obvious |
| 7 | **Repeatable automation** | Manual each time | No folder watch / default Open With habit |
| 8 | **Post-convert handoff** | Find `.md`, open, select all, paste | Missing “Convert & paste into frontmost app” |

---

## 4. What ParseKit already ships (codebase)

### Shipped low-friction surfaces

| Surface | Behavior |
|---------|----------|
| **Menu bar tray** | Left-click toggles popover; right-click: Open, **Parse Clipboard File → Copy Markdown**, Quit |
| **Drag & drop** | Popover drop zone for files/folders |
| **Batch UI** | Input/output, format, OCR, Run Parse, progress, history, copy last file, open in Finder |
| **Launch at login** | Settings |
| **Floating progress HUD** | Default on; background batches |
| **Finder Quick Actions** | “Parse to Markdown with ParseKit” + “Replace Original” (Automator → `~/Library/Services/`) |
| **Global hotkey** | Default **⌃⇧M** — Finder selection → background parse, *or* clipboard file path → Markdown on clipboard |
| **Auto-convert on Copy** | Opt-in pasteboard watcher (default **off**) |
| **CLI** | Bundled `parsekit-cli`; convert/batch; agent skill docs |
| **Onboarding** | Install-to-Applications, Gatekeeper fix, pick output folder, first-batch checklist |
| **Updater** | GitHub Releases check + install |

### Explicitly *not* built yet

- Raycast extension (brand mention only)  
- macOS Shortcuts / App Intents (README: planned later)  
- Share Extension / Share sheet  
- Native Services plugin beyond Automator workflows  

### Gaps vs marketing / discovery

| Issue | Reality |
|-------|---------|
| INSTALL docs say Settings → Finder | UI is **Settings → General → Installation & advanced** |
| Onboarding | Does **not** teach hotkey, clipboard convert, Finder QA, auto-convert, CLI |
| Auto-convert on copy | Shipped but **off** and almost invisible |
| Clipboard-to-Markdown | Easy to miss (tray *right-click*, hotkey fallback) |
| `clipboardConvertHint` locale string | Written but **unused** in UI |
| CLI on PATH | Binary in bundle; DMG users may not have `parsekit` on PATH |
| PRODUCT.md | Stale vs current features (hotkey, QA, HUD, tokens, CLI) |

### Underused powers (built, low discovery)

1. **⌃⇧M** select → Markdown (or output folder) without opening the panel  
2. **Auto-convert on Copy**  
3. **Tray right-click → Parse Clipboard**  
4. **Finder Quick Actions** + headless silent parse  
5. **Replace Original** workflow  
6. **CLI + agent skill**  
7. **Floating HUD + background-parse** mental model  
8. **Services menu** (same Automator workflows)  

### Highest-leverage product surface (do not rebuild)

Double down on:

```
Finder selection ──┐
Clipboard file  ──┼── hotkey / tray / auto-copy / Quick Action
open-queue      ──┘         │
                            ▼
              sidecar parse + notify / HUD / pbcopy
```

That pipeline is largely **already built**. The gap is **defaults + discovery**, not engines.

**Benchmark to own:** From file selected in Finder to Markdown in ChatGPT ≤ **2 keystrokes/clicks** (convert + paste), or **1** with Convert & Paste.

---

## 5. Happy paths to own (three contracts)

Design onboarding and marketing around these:

| Contract | Gesture | Result |
|----------|---------|--------|
| **A. Finder** | Select file(s) → Quick Action **Copy as Markdown** | ⌘V in chat |
| **B. Keyboard** | ⌃⇧M on Finder selection *or* file on clipboard | Clipboard MD; optional auto-paste |
| **C. Spatial** | Drop on menu bar / Dock | Chip → clipboard |

Everything else (file beside original, JSON for RAG, watch folders) is power-user mode.

---

## 6. Ranked opportunities (impact × effort)

### MUST-DO (high impact / medium or low effort)

#### 1. Primary output = clipboard Markdown by default
- Every entry point (Quick Action, hotkey, tray, Open With) ends with: *Markdown on pasteboard + brief HUD “Ready to paste (⌘V)”*  
- File write becomes secondary  
- **Why:** Collapses convert→open→select→copy into convert→paste  

#### 2. Quick Action: “Copy as Markdown” (default recommendation)
- Keep convert-beside-original / replace  
- Promote **Copy as Markdown** in onboarding and docs  
- **Why:** Finder is where files live; right-click is already in path  

#### 3. Teach the 10-second habit in onboarding
- Today: install + output folder + first batch  
- Add: **Select PDF → ⌃⇧M → ChatGPT → ⌘V** (GIF)  
- **Why:** That habit *is* the product  

#### 4. Unbury what you built
- Move **Install Finder Quick Actions** to Settings page 1 or onboarding  
- Show hotkey + “Parse clipboard” together  
- Render unused `clipboardConvertHint`  
- Optional: “Install CLI to PATH” in Settings  

#### 5. Token savings as first-run wow (“token receipt”)
Example first conversion:

> This PDF: **28.4k → 11.1k tokens** (61% saved) · 3 tables recovered · never left your Mac  

- **Why:** Competitors are silent on cost; ChatGPT is silent on waste; this is your viral screenshot  

#### 6. Harden auto-convert-on-copy with trust UX
- Clear menu-bar state: On / Off / Smart (only PDF/Office/image UTIs)  
- Never touch password managers’ concealed clipboard types  
- Skip short strings; show what was converted  
- **Why:** Feature exists; without trust it stays off  

#### 7. “Paste into frontmost app” / Convert & Paste
- After convert: optional auto-paste into ChatGPT web, Claude, Notes  
- CleanShot/TextSniper energy  
- Prefer clipboard write; Accessibility only if needed for auto-paste  

#### 8. Open With + document UTIs
- Register for PDF, DOCX, PPTX, XLSX, images  
- Opening a file = convert + result HUD (clipboard)  

---

### SHOULD-DO (high impact / higher effort)

#### 9. Result chip (upgrade the HUD)
- Preview snippet · Copy · Paste · Save `.md` · Open in Obsidian/VS Code · Re-run options  
- Make HUD the default product surface, not a status toast  

#### 10. Raycast extension (optionally Alfred)
- Commands: Convert Selection / Convert Clipboard / Convert File  
- Thin shell over existing convert core  
- **Why:** ICP lives in Raycast  

#### 11. App Intents + Shortcuts catalog
- Convert File, Convert Folder, Convert Clipboard, Set Output Format  
- Enables Action Button, Shortcuts automations, future Siri discovery  

#### 12. Share Extension: “ParseKit → Markdown”
- Appear in Share from Finder, Preview, Safari PDFs, Mail  
- **Cost:** real Swift extension target + App Groups (not pure Tauri)  

#### 13. Quality wedge vs native upload (product demos)
- Multi-column PDF, scanned pages, complex tables → token-efficient MD/JSON  
- Integrations deliver the product; quality is the *reason* not to drag raw PDF  

---

### LATER (nice / niche / high cost)

| Item | Notes |
|------|--------|
| Watch folder (Hazel-lite) `~/ParseKit Inbox` | Great for RAG pipelines, not daily chat |
| Browser Share / Safari PDF | Partly covered by Share Extension |
| Finder Sync badges | High maintenance, low adoption |
| Control Center / widgets | After App Intents |
| Continuity Camera scan → MD | After OCR excellence |
| Direct “Send to ChatGPT/Claude” deep links | Fragile; prefer clipboard + paste |
| Apple Intelligence Writing Tools hooks | Wrong layer for document conversion |

### Explicit non-priorities (for now)

- Competing as a full Claude Code / folder-agent replacement  
- Building a full clipboard manager (integrate with Paste/Raycast)  
- Building chat-with-docs (AnythingLLM/ChatGPT own Q&A)  
- App Store–only distribution if sandbox kills best power features  

---

## 7. Competitive landscape (short)

### Where users actually drop files

| Destination | Why it wins |
|-------------|-------------|
| **ChatGPT web/desktop** | Already open; habit; attach and go |
| **Claude (+ Projects)** | Long-doc reasoning; sticky project knowledge |
| **Gemini + NotebookLM** | Drive/Workspace gravity; multi-source notebooks |
| **Cursor (`@file`)** | Devs live in editor; MD/source wins; PDFs often converted offline |
| **Notion AI / enterprise** | File already in system of record |

### Technical peers (parsers)

| Product | Positioning | Adoption engine |
|---------|-------------|-----------------|
| **Marker** | High-fidelity PDF→MD | OSS + demos + CLI |
| **Docling (IBM)** | Multi-format → structured → MD/JSON | Brand + `pip` + privacy/air-gap story |
| **MarkItDown (Microsoft)** | Lightweight multi-format → MD | Extreme virality, one-liner CLI, VS Code right-click |
| **unstructured.io** | ETL for unstructured data | Free tier → paid platform + connectors |
| **LlamaParse** | Cloud agentic parse | Free credits + LlamaIndex ecosystem |
| **Mathpix Snip** | STEM OCR → LaTeX/MD | **Hotkey → clipboard** freemium habit |

### Strategic read

| Wins | How |
|------|-----|
| MarkItDown / Docling / Marker | *Developer discovery* (GitHub, pip, demos) |
| LlamaParse / Unstructured | *Builder production* (API credits, free tier) |
| Mathpix / Raycast / menu-bar AI | *Daily habit* (hotkey → clipboard) |
| ChatGPT / Claude / NotebookLM | *End-user Q&A* (no pipeline) |

**ParseKit’s gap:** There is no dominant **consumer-grade, local-first, macOS-native “clean then paste”** product. Quality tools are Python CLIs; convenience tools are cloud chat; privacy tools are full RAG apps.

**Wedge:** OS-native packaging of parser quality — **not another Python library, not another chat app.**

### Adoption tactics that work

| Tactic | Who does it well |
|--------|------------------|
| Generous free tier / free pages | LlamaParse, Unstructured |
| One-liner install | MarkItDown, Docling |
| GitHub + demo video | Marker, MarkItDown |
| Editor right-click | MarkItDown VS Code extension |
| Hotkey / Snip | Mathpix |
| Ecosystem SDK lock-in | LlamaIndex + LlamaParse |
| Desktop “download and run” | AnythingLLM |
| MCP / agent hooks | Emerging agent distribution |

### “Install in 30 seconds” rank (non-dev → power user)

1. DMG / App Store / Setapp  
2. `brew install --cask parsekit`  
3. Raycast extension  
4. Services / Quick Actions / Share sheet  
5. CLI `parsekit convert`  
6. MCP server for Claude Desktop / Cursor  

---

## 8. When power users prefer local pre-processing

| Reason | Detail |
|--------|--------|
| **Token / cost savings** | Layout noise, headers/footers, bad reading order inflate tokens; clean MD often cuts a large fraction and improves answers |
| **Privacy / compliance** | Contracts, health, finance, unreleased research — local first |
| **Hard docs** | Multi-column papers, tables, equations, scans — chat upload is inconsistent |
| **Bulk / batch** | Chat quotas break “clean 200 PDFs” |
| **Offline / air-gapped** | Real for some enterprises |
| **Better agent/RAG inputs** | Structured MD/JSON beats page dumps |

### What makes preference *automatic* (not intentional)

| Gesture | Automatic behavior |
|---------|-------------------|
| Select files / copy PDF | Hotkey → MD on clipboard + token estimate |
| Drop on menu bar / dock | Batch convert |
| Finder Quick Action | “Clean for AI” → `.md` or clipboard |
| Watch folder Inbox | Default-on folder action |
| Paste into chat | User pastes MD (habit after first token wow) |
| Raycast command | Fuzzy file → convert → copy |

**Packaging principle:** Sell **clipboard + hotkey + folder**, not “document intelligence platform.” Quality is the engine; **latency to clipboard** is the product.

---

## 9. Five ranked product wedges

### Rank 1 — “Clean for AI” clipboard hotkey (Mathpix × MarkItDown for general docs)

| | |
|--|--|
| **User job** | About to ask ChatGPT/Claude about this PDF; max quality per token in one shortcut |
| **Friction removed** | Raw upload token tax, weak tables, privacy anxiety |
| **GTM** | Product Hunt + demos (before/after tokens); Raycast; freemium daily snips |
| **MVP** | Menu bar; hotkey on selection or clipboard file/image → MD; token estimate; Copy/Save; no account |
| **Why it sticks** | Muscle memory; visible token save; last step before every AI paste |

### Rank 2 — Finder “Clean for AI” + drop folder (batch inbox)

| | |
|--|--|
| **User job** | Pile of PDFs → clean MD/JSON corpus for Projects / NotebookLM / Cursor / local RAG |
| **Friction removed** | Per-file upload limits, inconsistent extract, Python batch scripts |
| **GTM** | SEO + YouTube “100 PDFs to Claude Project”; research communities |
| **MVP** | Quick Action + optional `~/ParseKit/Inbox` watch; `Out/*.md`; progress; error report |
| **Why it sticks** | Folder habit + corpus investment (switching cost) |

### Rank 3 — Cursor / VS Code / Claude Code “docs pack”

| | |
|--|--|
| **User job** | Specs/RFCs/PDFs as first-class `@file` context |
| **Friction removed** | Broken PDF blobs; manual MarkItDown |
| **GTM** | Marketplace; brew CLI; MCP listing |
| **MVP** | CLI + thin extension; convert → MD next to source; optional `parsekit watch` |
| **Why it sticks** | Daily coding loop; team repos commit cleaned docs |

### Rank 4 — Privacy pre-flight (B2B-leaning, still Mac-native)

| | |
|--|--|
| **User job** | “AI help, but this file cannot leave the machine as a raw PDF” |
| **Friction removed** | Upload anxiety / policy |
| **GTM** | Privacy/security content; local LLM stacks |
| **MVP** | Hard local-only mode; optional redaction; sanitized MD for cloud chat; audit “never uploaded original” |
| **Why it sticks** | Documented compliance step; higher willingness to pay |

### Rank 5 — Agent/MCP parse tool

| | |
|--|--|
| **User job** | Agents call `parsekit` instead of stuffing raw PDFs into context |
| **Friction removed** | Every agent re-implements PDF parsing |
| **GTM** | MCP directories, Claude Desktop snippets, “local LlamaParse” content |
| **MVP** | Local MCP + stable JSON schema + CLI exit codes |
| **Why it sticks** | Platform gravity; treat as Phase 2 on top of Rank 1–2 |

### Ranked summary

| Rank | Wedge | Frequency | Differentiation |
|:----:|-------|:---------:|-----------------|
| 1 | Clipboard hotkey “Clean for AI” | Daily | OS habit + token UI |
| 2 | Finder + drop-folder batch | Weekly | Default-on automation |
| 3 | Editor/CLI docs pack | Daily (dev) | Agent-ready MD in repo |
| 4 | Privacy pre-flight | Event-driven | Trust / compliance |
| 5 | MCP/agent tool | High leverage | Infrastructure |

---

## 10. macOS integration map

### In-path (highest leverage)

| Integration | Intercepts | Notes for Tauri |
|-------------|------------|-----------------|
| **Share Extension** | Finder, Preview, Safari, Mail | Needs **native Swift** extension + App Groups |
| **Services** | Right-click Services | Low effort if Quick Actions already work |
| **Quick Actions** *(you have)* | Finder toolbar / right-click | Make **Copy as Markdown** primary |
| **Open With + UTIs** | Double-click / Open With | Converter destination |
| **App Intents / Shortcuts** | Shortcuts, Siri, Spotlight, Action Button | Swift bridge; multiplies surfaces |
| **Clipboard path** *(you have)* | After Copy | Privacy-sensitive |
| **Global hotkey + HUD** *(you have)* | Keyboard-first | Optimize Finder selection + clipboard file |
| **Drag to Dock / menu bar** | Spatial | Dock drop underused |

### Patterns to steal from successful Mac utilities

| App | Pattern | Steal for ParseKit |
|-----|---------|-------------------|
| **CleanShot** | Capture → floating result → auto clipboard | Result chip after convert |
| **TextSniper** | Hotkey → region → text on clipboard | Convert success = clipboard |
| **Paste / Raycast clipboard** | Live on the paste gesture | Last conversion always one paste away |
| **Raycast** | Keyboard, extensions, no app switch | Official extension |
| **Hazel** | Folder rules, zero UI | Optional watch folders |
| **Mathpix** | Hotkey → clipboard freemium | Daily habit model |
| **Obsidian** | Vault as destination | Optional “save to vault / daily note” |

### Risks

| Risk | Mitigation |
|------|------------|
| **Clipboard privacy / trust** | Opt-in auto-convert; UTI filters; visible “listening” indicator; prefer user-triggered convert |
| **Concealed clipboard types** | Never convert password-manager secrets (`org.nspasteboard.ConcealedType`) |
| **App Store sandbox** | Dual channel: notarized direct for power features; narrower App Store build if needed |
| **TCC / Accessibility** | Prefer pasteboard write + user paste over synthetic paste |
| **Tauri + extensions** | Budget thin Swift “Mac Integration” target for Share / App Intents |
| **Value proposition failure** | Lead with scanned PDFs, tables, tokens, privacy — integrations are delivery, not the wedge |
| **Drag-to-chat habit** | Don’t fight upload for every binary; win **paste-text** + **local/RAG/privacy** workflows |

---

## 11. Positioning & packaging

### Positioning line

> **Local PDF cleaning so AI sees structure, not sludge — without sending the file anywhere first.**

Not: “Another document converter.”  
Not: “Chat with your PDFs.”

### Landing / README proof points

1. **Token receipt** before/after  
2. **Finder → paste** GIF (≤2s loop)  
3. **Privacy badge** (never uploaded)  
4. **Hard docs** (scanned / multi-column / tables) where chat upload fails  

### Cross-cutting packaging

1. **Hero metric on first run:** token delta + tables recovered  
2. **Dual install:** Menu bar app (wedges 1–2, 4) + brew CLI/MCP (3, 5); same engine  
3. **Free tier that creates habit:** N pages/day free; Pro unlocks OCR/scans, large batch, JSON schema  
4. **Do not build chat** — own the 3 seconds before paste/upload  
5. **Default outputs by destination:**  
   - Claude/ChatGPT paste → compact MD  
   - Projects / NotebookLM → multi-file MD  
   - Cursor → `docs/parsed/*.md`  
   - RAG → JSON chunks  
6. **Viral loop:** Shareable before/after “token receipt” image (stats only, not document content)  

### Competitive gaps ParseKit can own

| Gap | Incumbents leave open |
|-----|----------------------|
| Consumer Mac UX on Docling/Marker-class quality | OSS stays in terminal |
| Token-savings as product surface | MarkItDown is silent on cost |
| Pre-Claude/ChatGPT step, not competing chat | AnythingLLM is full app (overkill) |
| Batch + privacy without Python | Cloud parsers want your pages |
| Hotkey → clipboard for *general* docs | Mathpix owns equations, not contracts |

---

## 12. 90-day execution plan

| Phase | Ship |
|-------|------|
| **Weeks 1–2** | Default path = clipboard MD + HUD “Ready to paste”; Quick Action **Copy as Markdown**; onboarding GIF (Finder → hotkey/QA → ChatGPT); unbury Finder QA install |
| **Weeks 3–4** | Token receipt UI; Convert & Paste (frontmost); Open With UTIs; auto-convert trust modes (On/Off/Smart) |
| **Month 2** | Result chip (Copy / Paste / Save / format); Raycast extension; CLI PATH install in Settings |
| **Month 3** | Share Extension (Swift); App Intents; polish Dock drop; optional watch folder for RAG users |

### Leverage order (build on existing code first)

1. Discovery of **hotkey + clipboard path**  
2. Promote **Finder Quick Actions** to first-class  
3. Optional **install CLI to PATH** in Settings  
4. Improve **auto-convert** trust (keep opt-in)  
5. **Only later:** Shortcuts / App Intents / Raycast / Share as thin shells over the same convert core  

### Avoid rebuilding

- New tray UI paradigms for their own sake  
- Share Extension before clipboard-default path is productized  
- Parallel convert engines for “integrations”  
- Full chat-with-docs product  

---

## 13. Concrete “next build” (highest ROI)

If shipping only one product slice next:

### “Copy as Markdown” + habit + token receipt

1. **Finder Quick Action:** Copy as Markdown (clipboard primary)  
2. **Onboarding step:** Select PDF → ⌃⇧M → paste in ChatGPT (GIF)  
3. **First convert UI:** Token before/after + “Ready to paste”  
4. **Settings:** Finder QA install on page 1; hotkey + clipboard hints visible  

**No new platforms required.** Uses engines already in the repo.

---

## 14. Bottom line

| Truth | Implication |
|-------|-------------|
| Users will keep dropping files into ChatGPT/Claude/NotebookLM/Cursor | You cannot win “open PDF and ask” as the primary job |
| Local parsers win *builders*; chat wins *end users* | ParseKit must be **OS habit**, not a library or a chat app |
| You already have tray + QA + hotkey + clipboard + HUD + CLI | Adoption moves when **defaults and discovery** match that power |
| Power users skip intentional pre-processing | Preference becomes sticky when token/privacy/quality appear **without an extra deliberate workflow** |

**ParseKit should not ask users to “choose a better parser.”**  
It should make cleaning the **default gesture** on macOS:

1. Hotkey → clipboard (daily chat)  
2. Folder → batch corpus (research/RAG)  
3. Then editor/MCP (developers/agents)  
4. Paid privacy workflows (sensitive docs)  

---

## 15. Appendix: three research tracks used

This document synthesizes:

1. **macOS friction & integration research** — Share Extension, Services, Quick Actions, App Intents, clipboard privacy, patterns from CleanShot/TextSniper/Raycast/Hazel  
2. **Competitive & GTM research** — ChatGPT/Claude/NotebookLM/Cursor habits; Marker/Docling/MarkItDown/LlamaParse/Mathpix; adoption tactics  
3. **ParseKit codebase exploration** — What is shipped vs buried vs not built (tray, hotkey, clipboard, Finder QA, CLI, onboarding)  

Project path: `<local-path>`  
Repo: `https://github.com/harshabala/PareseKit`

---

*End of note — for review and prioritization.*
