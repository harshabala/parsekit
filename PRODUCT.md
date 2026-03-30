# ParseDock – Product Brief

## Overview
ParseDock is a lightweight, native-feeling macOS menu-bar app that batch-converts folders of PDFs, Office docs, and images into clean text, Markdown, or JSON — ready to drag into ChatGPT, Claude, Grok, local RAG, or any knowledge tool. It uses LiteParse (@llamaindex/liteparse) under the hood but hides all complexity.

## One-Line Description
ParseDock is the fastest way to turn any folder of documents into LLM-ready files with zero terminal and zero cloud.

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
- 100% local (only LiteParse's optional first-run Tesseract download).
- Native macOS look & feel (dark mode, SF Symbols, accessibility, haptics).
- Zero telemetry, zero network except LiteParse necessities.
- Minimal CPU/memory; smart concurrency (max 4-6 workers on M-series).
- Signed & notarized .app build ready.
