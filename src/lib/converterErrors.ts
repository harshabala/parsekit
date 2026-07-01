/** True when a parse error likely means a missing optional converter (Office / images). */
export function isConverterDependencyError(message: string | null | undefined): boolean {
  if (!message) return false;
  const lower = message.toLowerCase();
  return (
    lower.includes("imagemagick") ||
    lower.includes("libreoffice") ||
    lower.includes("brew install")
  );
}

export type SettingsTab = "general" | "file-support";