import { invoke } from "@tauri-apps/api/core";
import type { TokenStatsPeriod } from "./store";

export interface FileTypeStats {
  files: number;
  tokens_saved: number;
}

export interface TokenEvent {
  ts: string;
  file_type: string;
  tokens_saved: number;
  pages_unlocked: number;
}

export interface TokenStats {
  total_files_converted: number;
  total_tokens_saved: number;
  total_pages_unlocked: number;
  total_documents_unlocked: number;
  by_file_type: Record<string, FileTypeStats>;
  events: TokenEvent[];
}

export interface RecordTokenSavingsInput {
  fileType: string;
  tokensSaved: number;
  pagesUnlocked?: number;
  documentsUnlocked?: number;
}

export async function getTokenStats(): Promise<TokenStats> {
  return invoke<TokenStats>("get_token_stats");
}

export async function resetTokenStats(): Promise<TokenStats> {
  return invoke<TokenStats>("reset_token_stats");
}

export async function recordTokenSavings(
  input: RecordTokenSavingsInput,
): Promise<TokenStats> {
  return invoke<TokenStats>("record_token_savings", {
    fileType: input.fileType,
    tokensSaved: Math.max(0, input.tokensSaved),
    pagesUnlocked: input.pagesUnlocked ?? 0,
    documentsUnlocked: input.documentsUnlocked ?? 0,
  });
}

/** Inclusive start, exclusive end — matches calendar period helpers below. */
export function filterEventsInRange(
  events: TokenEvent[],
  start: Date,
  end: Date,
): TokenEvent[] {
  const startMs = start.getTime();
  const endMs = end.getTime();
  return events.filter((event) => {
    const ts = new Date(event.ts).getTime();
    return !Number.isNaN(ts) && ts >= startMs && ts < endMs;
  });
}

export function sumTokensInRange(
  events: TokenEvent[],
  start: Date,
  end: Date,
): number {
  return filterEventsInRange(events, start, end).reduce(
    (sum, event) => sum + event.tokens_saved,
    0,
  );
}

export function sumPagesUnlockedInRange(
  events: TokenEvent[],
  start: Date,
  end: Date,
): number {
  return filterEventsInRange(events, start, end).reduce(
    (sum, event) => sum + event.pages_unlocked,
    0,
  );
}

export function startOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

export function endOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth() + 1, 1);
}

/** ISO week: Monday 00:00 local time through the following Monday. */
export function startOfWeek(date: Date): Date {
  const start = new Date(date);
  start.setHours(0, 0, 0, 0);
  const day = start.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  start.setDate(start.getDate() + diff);
  return start;
}

export function endOfWeek(date: Date): Date {
  const end = startOfWeek(date);
  end.setDate(end.getDate() + 7);
  return end;
}

export function tokensSavedThisMonth(
  stats: TokenStats,
  now: Date = new Date(),
): number {
  return sumTokensInRange(stats.events, startOfMonth(now), endOfMonth(now));
}

export function tokensSavedThisWeek(
  stats: TokenStats,
  now: Date = new Date(),
): number {
  return sumTokensInRange(stats.events, startOfWeek(now), endOfWeek(now));
}

export function pagesUnlockedThisMonth(
  stats: TokenStats,
  now: Date = new Date(),
): number {
  return sumPagesUnlockedInRange(
    stats.events,
    startOfMonth(now),
    endOfMonth(now),
  );
}

/** Rough tokens per long ChatGPT message — relatable comparison only. */
export const LONG_CHATGPT_MESSAGE_TOKENS = 800;

export function approximateChatGptMessages(tokens: number): number {
  if (tokens <= 0) return 0;
  return Math.max(1, Math.round(tokens / LONG_CHATGPT_MESSAGE_TOKENS));
}

export function formatTokenCount(value: number): string {
  return Math.max(0, Math.floor(value)).toLocaleString();
}

export function tokensForPeriod(
  stats: TokenStats,
  period: TokenStatsPeriod,
  now: Date = new Date(),
): number {
  if (period === "lifetime") {
    return stats.total_tokens_saved;
  }
  return tokensSavedThisMonth(stats, now);
}

export interface SidecarTokenSavingsEvent {
  file_type?: string;
  tokens_saved?: number;
  pages_unlocked?: number;
  documents_unlocked?: number;
}

/** Record savings from a sidecar `token_savings` line; no-op when file type is missing. */
export async function recordTokenSavingsFromSidecarEvent(
  event: SidecarTokenSavingsEvent,
): Promise<TokenStats | null> {
  const fileType = event.file_type?.trim();
  if (!fileType) return null;
  return recordTokenSavings({
    fileType,
    tokensSaved: event.tokens_saved ?? 0,
    pagesUnlocked: event.pages_unlocked ?? 0,
    documentsUnlocked: event.documents_unlocked ?? 0,
  });
}