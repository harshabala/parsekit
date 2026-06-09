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

/** macOS often uses /var vs /private/var for the same file — match either form. */
export function pathsMatchForProgress(a: string, b: string): boolean {
  if (a === b) return true;
  const norm = (p: string) => p.replace(/\/+/g, "/").replace(/\/$/, "");
  const na = norm(a);
  const nb = norm(b);
  if (na === nb) return true;
  const stripPrivate = (p: string) =>
    p.startsWith("/private/var/") ? `/var/${p.slice("/private/var/".length)}` : p;
  return stripPrivate(na) === stripPrivate(nb);
}

function findFileIndex(files: FileProgress[], event: ParseProgressEvent): number {
  const sourcePath = event.sourcePath;
  const displayName = event.file || (sourcePath ? fileBaseName(sourcePath) : "");
  if (sourcePath) {
    const byPath = files.findIndex((f) => pathsMatchForProgress(f.id, sourcePath));
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

export interface BatchSettleMessages {
  /** Files that were actively parsing when the batch stopped. */
  parsing?: string;
  /** Files never started (still queued). */
  pending?: string;
}

/**
 * Finalize all non-terminal rows when the batch ends early (cancel, stall, sidecar exit).
 * Ensures nothing stays in "Waiting" / "Parsing" forever.
 */
export function settleBatchOnStop(
  files: FileProgress[],
  messages: BatchSettleMessages
): FileProgress[] {
  const parsingMsg = messages.parsing ?? messages.pending ?? "Batch stopped.";
  const pendingMsg = messages.pending ?? messages.parsing ?? parsingMsg;

  return files.map((f) => {
    if (f.status === "parsing") {
      return { ...f, status: "error" as const, error: parsingMsg };
    }
    if (f.status === "pending") {
      return { ...f, status: "error" as const, error: pendingMsg };
    }
    return f;
  });
}

/** @deprecated Use settleBatchOnStop */
export function settleInFlightOnAbort(
  files: FileProgress[],
  message: string
): FileProgress[] {
  return settleBatchOnStop(files, { parsing: message, pending: message });
}

/** After a successful `done` event, clear any rows that never received a final status. */
export function settleRemainingOnDone(
  files: FileProgress[],
  message: string
): FileProgress[] {
  return settleBatchOnStop(files, { parsing: message, pending: message });
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