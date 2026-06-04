import type { FileProgress, FileStatus } from "./types";
import { fileBaseName } from "./supportedExtensions";

export interface ParseProgressEvent {
  type: "progress";
  file?: string;
  sourcePath?: string;
  status?: string;
  path?: string;
  error?: string;
}

export interface ParseProgressApplyResult {
  files: FileProgress[];
  /** Most recently started parse (by source path), for the “now parsing” banner. */
  lastParsingId: string | null;
}

function mapSidecarStatus(raw: string | undefined): FileStatus {
  if (raw === "completed") return "done";
  if (raw === "parsing") return "parsing";
  if (raw === "error") return "error";
  if (raw === "skipped") return "skipped";
  return "pending";
}

function findFileIndex(files: FileProgress[], event: ParseProgressEvent): number {
  const sourcePath = event.sourcePath;
  const displayName = event.file || (sourcePath ? fileBaseName(sourcePath) : "");
  if (sourcePath) {
    const byPath = files.findIndex((f) => f.id === sourcePath);
    if (byPath !== -1) return byPath;
  }
  if (!displayName) return -1;
  // Prefer the oldest still-active row with this basename (FIFO within duplicates).
  return files.findIndex(
    (f) =>
      f.name === displayName &&
      (f.status === "pending" || f.status === "parsing")
  );
}

/** Apply a sidecar progress event; returns a new array for reliable Svelte reactivity. */
export function applyParseProgressEvent(
  files: FileProgress[],
  event: ParseProgressEvent,
  lastParsingId: string | null = null
): ParseProgressApplyResult {
  const sourcePath = event.sourcePath;
  const displayName = event.file || (sourcePath ? fileBaseName(sourcePath) : "");
  const status = mapSidecarStatus(event.status);

  const existingIndex = findFileIndex(files, event);

  let nextFiles: FileProgress[];
  if (existingIndex !== -1) {
    nextFiles = files.map((f, i) =>
      i === existingIndex
        ? {
            ...f,
            status,
            outputPath: event.path ?? f.outputPath,
            error: event.error,
          }
        : f
    );
  } else if (sourcePath) {
    nextFiles = [
      {
        id: sourcePath,
        name: displayName,
        status,
        outputPath: event.path,
        error: event.error,
      },
      ...files,
    ];
  } else {
    return { files, lastParsingId };
  }

  let nextLast = lastParsingId;
  if (status === "parsing" && sourcePath) {
    nextLast = sourcePath;
  } else if (nextLast && status !== "parsing") {
    const stillParsing = nextFiles.some(
      (f) => f.id === nextLast && f.status === "parsing"
    );
    if (!stillParsing) {
      const fallback = nextFiles.find((f) => f.status === "parsing");
      nextLast = fallback?.id ?? null;
    }
  }

  return { files: nextFiles, lastParsingId: nextLast };
}

/** Mark in-flight rows as failed when the batch stops unexpectedly. Pending rows stay pending. */
export function settleInFlightOnAbort(
  files: FileProgress[],
  message: string
): FileProgress[] {
  return files.map((f) =>
    f.status === "parsing"
      ? { ...f, status: "error" as const, error: message }
      : f
  );
}

/** Pick the row to highlight in the progress banner. */
export function resolvePrimaryParsingId(
  files: FileProgress[],
  lastParsingId: string | null
): string | null {
  if (lastParsingId) {
    const match = files.find(
      (f) => f.id === lastParsingId && f.status === "parsing"
    );
    if (match) return match.id;
  }
  return files.find((f) => f.status === "parsing")?.id ?? null;
}