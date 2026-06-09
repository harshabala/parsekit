import en from "../locales/en.json";
import zh from "../locales/zh.json";
import es from "../locales/es.json";

export type AppLocale = "en" | "zh" | "es";

export const SUPPORTED_LOCALES: readonly AppLocale[] = ["en", "zh", "es"];

const catalogs: Record<AppLocale, Record<string, unknown>> = { en, zh, es };

// Module-private reactive state. Svelte 5 forbids exporting a reassigned $state,
// so callers read it through getLocale() (reactive) and mutate it via initLocale().
let localeValue = $state<AppLocale>("en");

export function getLocale(): AppLocale {
  return localeValue;
}

export function normalizeLocale(value: unknown): AppLocale {
  if (value === "zh" || value === "es" || value === "en") {
    return value;
  }
  return "en";
}

/** Map legacy settings when only `ocrLanguage` existed (before separate UI locale). */
export function localeFromLegacyOcr(ocrLanguage: string): AppLocale {
  if (ocrLanguage.startsWith("chi")) {
    return "zh";
  }
  if (ocrLanguage === "spa") {
    return "es";
  }
  return "en";
}

export function applyDocumentLocale(code: AppLocale) {
  document.documentElement.lang = code === "zh" ? "zh-Hans" : code;
}

function lookup(table: Record<string, unknown>, key: string): string | undefined {
  const parts = key.split(".");
  let current: unknown = table;
  for (const part of parts) {
    if (!current || typeof current !== "object" || !(part in current)) {
      return undefined;
    }
    current = (current as Record<string, unknown>)[part];
  }
  return typeof current === "string" ? current : undefined;
}

export function t(
  key: string,
  vars?: Record<string, string | number>
): string {
  const active = localeValue;
  let text =
    lookup(catalogs[active], key) ??
    lookup(catalogs.en, key) ??
    key;

  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.replaceAll(`{${name}}`, String(value));
    }
  }
  return text;
}

export function initLocale(code: AppLocale) {
  localeValue = code;
  applyDocumentLocale(code);
}