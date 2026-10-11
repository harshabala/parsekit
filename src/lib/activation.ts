/**
 * ParseKit activation — single named event for product metrics.
 * Local-only; never uploaded.
 *
 * Spec PK-1: activated = first job with status success AND tokens estimate computed
 * (or at least one successful convert recorded).
 */
export const ACTIVATION_EVENT = "first_successful_convert_with_token_estimate" as const;

/** Settings key: ISO timestamp when activation first occurred. */
export const ACTIVATION_AT_KEY = "activatedAt";

/** Settings key: whether the optional destination prompt was answered or skipped. */
export const DESTINATION_PROMPT_KEY = "destinationPromptSeen";

/** Settings key for soft destination preference (local only). */
export const CONVERT_DESTINATION_KEY = "convertDestination";

export type ConvertDestination =
  | "claude"
  | "chatgpt"
  | "gemini"
  | "obsidian"
  | "other"
  | "skipped";

export function isActivated(activatedAt: string | number | null | undefined): boolean {
  if (activatedAt == null || activatedAt === "") return false;
  return true;
}

/**
 * Whether this convert should flip activation.
 * Requires at least one successful file and non-negative token estimate recorded
 * (tokens may be 0 for tiny files — still counts if success).
 */
export function qualifiesForActivation(args: {
  alreadyActivated: boolean;
  successCount: number;
  tokensSavedInBatch: number;
}): boolean {
  if (args.alreadyActivated) return false;
  return args.successCount >= 1 && args.tokensSavedInBatch >= 0;
}
