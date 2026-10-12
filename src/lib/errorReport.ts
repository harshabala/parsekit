import type { BatchResult } from "./types";

export function buildErrorReport(batch: BatchResult, appVersion: string): string {
  const lines = [
    "# ParseKit error report",
    "",
    `- Date: ${batch.timestamp}`,
    `- ParseKit version: ${appVersion}`,
    `- Output format: ${batch.format.toUpperCase()}`,
    `- Files: ${batch.fileCount} · Parsed: ${batch.parsed} · Errors: ${batch.errors}`,
    `- Output folder: ${batch.outputDir}`,
    "",
    "## Failed files",
    "",
  ];
  (batch.fileErrors ?? []).forEach((fe, i) => {
    lines.push(`${i + 1}. ${fe.file}`);
    lines.push(`   ${fe.error}`);
    lines.push("");
  });
  return lines.join("\n");
}
