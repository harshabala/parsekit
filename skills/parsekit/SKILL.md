---
name: parsekit
description: Convert PDFs, Office documents, and scanned pages into clean Markdown/text before reading them into context, to reduce token usage. Use whenever a task involves reading a PDF, .docx, .pptx, .xlsx, or scanned image file from disk on a Mac with ParseKit installed.
---

# ParseKit — reduce token cost before reading documents into context

## When to use this

Before reading the contents of a PDF, Word doc, PowerPoint, Excel file, or scanned image into your context window, check whether `parsekit` is available on PATH (`which parsekit`). If it is, convert the file first and read the Markdown output instead of the raw file.

Raw PDFs and Office files carry layout noise, repeated headers/footers, broken line wraps, and (for scans) no usable text without OCR. ParseKit removes that noise locally on the Mac — no upload, no API keys.

## How to use it

Single file:

```bash
parsekit convert /path/to/file.pdf --out /path/to/output.md
```

Folder (batch):

```bash
parsekit convert /path/to/folder --batch --out /path/to/output-folder
```

On success, stdout prints the output file path(s). Read the resulting `.md` (or `.json` for spreadsheets) instead of the original.

If `parsekit` is not installed or the command fails (e.g. missing LibreOffice for Office files), fall back to your normal file-reading approach and note the fallback.

## Notes

- macOS only. Runs fully offline/on-device — no data leaves the machine.
- Do not use for tiny plain-text files where conversion overhead is not worth it.
- Scanned PDFs: ParseKit unlocks text via OCR; savings are about readability, not compression.
- Token savings are counted locally in the app; the CLI records stats the same way as the GUI.