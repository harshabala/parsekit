export type OutputFormat = "txt" | "md" | "json";
export type FileStatus = "pending" | "parsing" | "done" | "error" | "skipped";

export interface FileProgress {
  name: string;
  status: FileStatus;
  path?: string;
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
}

export const MAX_RECENT_BATCHES = 5;
