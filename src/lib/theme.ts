import type { ThemeMode } from "./types";

export const DEFAULT_THEME: ThemeMode = "system";

const THEME_MODES: ThemeMode[] = ["light", "dark", "system"];

export function normalizeThemeMode(value: unknown): ThemeMode {
  if (typeof value === "string" && THEME_MODES.includes(value as ThemeMode)) {
    return value as ThemeMode;
  }
  return DEFAULT_THEME;
}

export function applyTheme(mode: ThemeMode): void {
  document.documentElement.dataset.theme = mode;
}