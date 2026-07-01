import { invoke } from "@tauri-apps/api/core";
import { emitTo } from "@tauri-apps/api/event";
import type { BatchTokenSavings } from "./progress";
import type { FileProgress } from "./types";

export const PROGRESS_HUD_WINDOW_LABEL = "progress-hud";
export const PROGRESS_HUD_SYNC_EVENT = "hud-sync";
export const PROGRESS_HUD_OPEN_FILE_SUPPORT_EVENT = "hud-open-file-support";
export const PROGRESS_HUD_DISMISS_MS = 5000;

export interface ProgressHudState {
  files: FileProgress[];
  total: number;
  isParsing: boolean;
  batchTokenSavings: BatchTokenSavings;
}

export function createEmptyHudState(): ProgressHudState {
  return {
    files: [],
    total: 0,
    isParsing: false,
    batchTokenSavings: {
      tokensSaved: 0,
      pagesUnlocked: 0,
      documentsUnlocked: 0,
    },
  };
}

export function hudFinishedCount(files: FileProgress[]): number {
  return files.filter(
    (f) => f.status === "done" || f.status === "error" || f.status === "skipped",
  ).length;
}

export function hudSuccessCount(files: FileProgress[]): number {
  return files.filter((f) => f.status === "done").length;
}

export function hudErrorCount(files: FileProgress[]): number {
  return files.filter((f) => f.status === "error").length;
}

export function hudProgressPercent(files: FileProgress[], total: number): number {
  if (total <= 0) return 0;
  return Math.round((hudFinishedCount(files) / total) * 100);
}

export function hudIsComplete(files: FileProgress[], total: number, isParsing: boolean): boolean {
  if (isParsing || total <= 0) return false;
  return hudFinishedCount(files) >= total;
}

export async function showProgressHudWindow(): Promise<void> {
  try {
    await invoke("show_progress_hud");
  } catch (e) {
    console.warn("[progressHud] show failed", e);
  }
}

export async function hideProgressHudWindow(): Promise<void> {
  try {
    await invoke("hide_progress_hud");
  } catch (e) {
    console.warn("[progressHud] hide failed", e);
  }
}

export async function syncProgressHud(
  state: ProgressHudState,
  retries = 4,
): Promise<void> {
  for (let attempt = 0; attempt < retries; attempt += 1) {
    try {
      await emitTo(PROGRESS_HUD_WINDOW_LABEL, PROGRESS_HUD_SYNC_EVENT, state);
      return;
    } catch (e) {
      if (attempt === retries - 1) {
        console.warn("[progressHud] sync failed", e);
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 60 * (attempt + 1)));
    }
  }
}