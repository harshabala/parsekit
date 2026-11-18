# ParseKit: North Star and the road to 10/10

Written 2026-11-13 from a review of the repository, `docs/TASKS.md` and
`docs/benchmark-results.md`.

The owner has not yet described their own complaint about this project, so the North Star below
is proposed and should be confirmed.

Read "Rules for agents" first, then take the first workstream whose "Done when" is not yet true.

## What it is

A menu bar app that turns PDFs, Office documents and images into clean Markdown on the Mac.
Offline, no accounts, no keys. It is public and has releases (0.2.12).

## North Star (proposed)

**Documents converted that needed no fixing by hand afterwards.**

## What 10/10 looks like: three scenarios

1. **Faithful.** A real report with headings, lists, tables, footnotes and figures comes out as
   Markdown with the same structure and reading order, and tables that are still tables.
2. **Scans too.** A scanned or photographed page comes out as correct text, including multi-column
   layouts.
3. **Drop and done.** Drag in a folder of mixed files; a few seconds later the Markdown is where
   the owner wants it, with a clear note on anything that did not convert well.

## Where it is today (purpose score: 7/10)

2026-11-18: was 6. The extra point is honest token numbers. The published fixtures
still show no reduction. Quality is still unmeasured.

True today:
- Works and is released. Conversion through a bundled sidecar, with optional OCR.
- First-run flow, a scoreboard of results, failure paths and success rate (tasks PK-1 to PK-5).
- A check of which helper tools are available on the Mac.

Gaps:
- **Quality has never been measured.** The only benchmark counts tokens on five tiny synthetic
  files. Nothing measures whether headings, tables and reading order are right.
- **The headline number does not hold up.** The app shows token savings, but the benchmark
  records a saving of 0 % on every fixture, and ParseKit's output is larger than the naive
  baseline in each case (a spreadsheet goes from 19 tokens to 641). Either the claim or the
  measurement needs to change.
- Spreadsheets always come out as JSON, not Markdown tables.
- `docs/` holds duplicate files ending in " 2.md", which are copies made by iCloud sync conflicts.
- A screenshots branch is unmerged.

## Workstreams, in order

### 0. One clean base
- Merge or close `docs/parsekit-screenshots`. Compare each " 2.md" duplicate with its original,
  keep the right one and delete the other.
- **Done when:** one checkout on `master`, no duplicate documents, GitHub equals local.
- 2026-11-18: not done. Tracked files have no ` 2.md` copies; the local copies are
  gitignored and the ones that differ only put personal paths back. They were
  not kept. `docs/parsekit-screenshots` is behind `master`, which already has
  the gallery commit `dc38b40`. Do not merge that branch. Not merged to
  `master`, and not pushed. Not a scoring point.

### 1. A quality benchmark (before any conversion change)
- A set of at least 30 real documents the owner may share or that are freely licensed: reports
  with tables, academic papers in two columns, slide decks, scans, photos of pages, spreadsheets,
  forms, and non-English text.
- Hand-checked expected Markdown for each, and a scorer for heading structure, reading order,
  table cells, and text accuracy.
- **Done when:** the baseline scores exist for every document type.

### 2. Close the worst gaps
- Work down the benchmark from the worst type. Likely candidates: tables, multi-column order,
  spreadsheets as Markdown tables with JSON as an option, headers and footers repeated on every
  page, and hyphenated line breaks.
- **Done when:** every document type meets a target agreed with the owner.

### 3. Tell the truth about tokens
- Re-run the token benchmark on the real document set. Show a saving only where it is real, and
  compare against what people actually paste into an AI tool. If structure is the value and not
  size, say that instead.
- **Done when:** every number shown in the app and the README is reproduced by a script.
- 2026-11-18: the README no longer says "~4 chars/token". It states `cl100k_base`,
  `max(0, baseline − output)`, the 800-token ChatGPT approximation, and the
  published 0.0% fixture deltas. `node scripts/reproduce_published_numbers.mjs`
  prints those figures from the benchmark files and the source constant.
  The settings hint no longer says tokens were avoided. Live counters are
  still sums of on-device events, not a new measurement. This is the honest
  token-numbers point. Score 6 → 7.

### 4. Know when it went badly
- A confidence note per file: low OCR confidence, a table that could not be kept, pages skipped.
  Show the source page beside the output for quick checking.
- **Done when:** every benchmark document that scores badly is also flagged to the user.

### 5. Feel and flow
- Drop anywhere, sensible default destination, a watch folder if the owner wants one. Motion and
  states per the existing design notes.
- **Done when:** the owner accepts a screen recording of scenario 3.

### 6. Release
- Signed and notarized build; update the README screenshots.
- **Done when:** the new version is on the releases page.

## Rules for agents

- Files never leave the Mac. Network only for optional updates and first-time OCR language packs.
- A quality claim is made only with benchmark scores before and after.
- Never show a number in the product that a script cannot reproduce.
- No file names or paths in any stored metric.

## Needs the owner

- Confirming the North Star and what bothers them most about ParseKit today.
- Supplying or approving the real document set; agreeing targets per type.
- Signing credentials for release.

## Scoring

One point each, ten in all: benchmark and baseline (1); tables (2); reading order (1); scans (1);
spreadsheets (1); honest token numbers (1); bad conversions flagged (1); owner accepts the flow
(1); released (1).

Today: 7. The headline saving is no longer claimed. Quality is still unmeasured.
