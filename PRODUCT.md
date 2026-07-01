# ParseKit – Product Brief

## Overview
ParseKit is a lightweight, native-feeling macOS menu-bar app that batch-converts folders of PDFs, Office docs, and images into clean text, Markdown, or JSON — ready to drag into ChatGPT, Claude, Grok, local RAG, or any knowledge tool. It uses LiteParse v2 (Rust-native) under the hood but hides all complexity.

## One-Line Description
ParseKit turns a folder of documents into text files you can paste anywhere — no Terminal, no cloud upload.

## Target Users
Knowledge workers, researchers, lawyers, consultants, and power users who live in PDFs/Office docs and want instant, private, local preprocessing before feeding LLMs or building RAG indexes.

## Core Jobs-to-be-Done
1. Quick ingest: folder of PDFs → clean Markdown/text files in seconds.
2. Routine batch processing: normalize incoming regulatory docs/contracts into a consistent output folder.
3. Pre-processing for vector DBs & local RAG: produce predictable, layout-aware output.

## Functional Requirements
- Menu-bar icon → elegant popover panel (Tauri tray + custom HTML window).
- Controls (all persisted):
  - Input folder picker (recursive).
  - Output folder picker.
  - Output format selector: Text (.txt), Markdown (.md), JSON (.json).
  - OCR toggle (default ON) + language selector (default "eng").
- One-click "Run Parse" button.
- Live progress: global bar + per-file list (Parsed / Skipped / Error).
- Recent batches (last 5) with one-click "Open in Finder".
- Post-batch: "Open Output Folder", "Copy last Markdown to clipboard".
- Smart Markdown output: # Title + ## Page N markers + --- separators.

## Non-Functional Requirements
- 100% local (LiteParse v2 native OCR; optional first-run Tesseract language data).
- Native macOS look & feel (dark mode, SF Symbols, accessibility, haptics).
- Zero telemetry. Network use is limited to optional GitHub release checks (updater) and first-run OCR language data downloads when needed.
- Minimal CPU/memory; smart concurrency (max 4-6 workers on M-series).
- Ad-hoc signed `.app` builds (see `docs/RELEASING.md`); notarization is planned for public distribution.
