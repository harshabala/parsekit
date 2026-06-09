export type OutputFormat = "txt" | "md" | "json";
export type ThemeMode = "light" | "dark" | "system";
export type { AppLocale } from "./i18n.svelte";
export type FileStatus = "pending" | "parsing" | "done" | "error" | "skipped";

export interface FileProgress {
  /** Full input path — unique key for progress updates */
  id: string;
  /** Display name (basename) */
  name: string;
  status: FileStatus;
  /** Output file path when done/skipped */
  outputPath?: string;
  error?: string;
}

export interface BatchResult {
  id: string;
  timestamp: string;
  inputDir: string;
  outputDir: string;
  format: OutputFormat;
  fileCount: number;
  parsed: number;
  errors: number;
  /** Present when batch used explicit file selection (for re-run). */
  sourcePaths?: string[];
}

export const MAX_RECENT_BATCHES = 10;
