/** Tesseract OCR language codes for text inside input documents (not UI, not output format). */
export type OcrLanguageCode =
  | "eng"
  | "chi_sim"
  | "spa"
  | "hin"
  | "tam"
  | "kan";

export interface OcrLanguageOption {
  code: OcrLanguageCode;
  /** Display name (native script where helpful). */
  label: string;
}

export const OCR_LANGUAGES: readonly OcrLanguageOption[] = [
  { code: "eng", label: "English" },
  { code: "chi_sim", label: "中文 (Simplified)" },
  { code: "spa", label: "Español" },
  { code: "hin", label: "हिन्दी (Hindi)" },
  { code: "tam", label: "தமிழ் (Tamil)" },
  { code: "kan", label: "ಕನ್ನಡ (Kannada)" },
] as const;

const VALID_CODES = new Set<string>(OCR_LANGUAGES.map((o) => o.code));

export function isKnownOcrLanguage(value: string): boolean {
  return VALID_CODES.has(value);
}

export function normalizeOcrLanguage(value: unknown): OcrLanguageCode {
  if (typeof value === "string" && VALID_CODES.has(value as OcrLanguageCode)) {
    return value as OcrLanguageCode;
  }
  if (typeof value === "string" && value.startsWith("chi")) {
    return "chi_sim";
  }
  return "eng";
}

export function ocrLanguageLabel(code: OcrLanguageCode): string {
  return OCR_LANGUAGES.find((o) => o.code === code)?.label ?? code;
}