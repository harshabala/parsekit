/** Keep in sync with SUPPORTED_EXTENSIONS in src-tauri/src/lib.rs */
export const SUPPORTED_EXTENSIONS = [
  "pdf", "doc", "docx", "docm", "odt", "rtf", "ppt", "pptx", "pptm", "odp",
  "xls", "xlsx", "xlsm", "ods", "csv", "tsv", "png", "jpg", "jpeg", "gif",
  "bmp", "tiff", "tif", "webp", "svg",
] as const;

const EXTENSION_SET = new Set<string>(SUPPORTED_EXTENSIONS);

export function isSupportedFilePath(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase();
  return !!ext && EXTENSION_SET.has(ext);
}

export function filterSupportedPaths(paths: string[]): string[] {
  return paths.filter(isSupportedFilePath);
}

export function fileBaseName(path: string): string {
  const parts = path.split(/[/\\]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}