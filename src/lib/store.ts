import { LazyStore } from "@tauri-apps/plugin-store";

const store = new LazyStore("settings.json");

/** Controls which window the quiet banner / period toggle uses. */
export type TokenStatsPeriod = "today" | "month" | "lifetime";

export const DEFAULT_TOKEN_STATS_PERIOD: TokenStatsPeriod = "today";

/** Avoid hanging the UI if the store IPC never responds (e.g. early webview init). */
const STORE_READ_TIMEOUT_MS = 3000;

async function withStoreTimeout<T>(
  op: () => Promise<T>,
  label: string,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () => reject(new Error(`${label} timed out after ${STORE_READ_TIMEOUT_MS}ms`)),
      STORE_READ_TIMEOUT_MS,
    );
  });
  try {
    return await Promise.race([op(), timeout]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

export async function getSetting<T>(key: string, defaultValue: T): Promise<T> {
  try {
    const val = await withStoreTimeout(
      () => store.get<T>(key),
      `getSetting(${key})`,
    );
    return val !== undefined ? val : defaultValue;
  } catch (error) {
    console.warn(`[ParseKit] getSetting(${key}) failed, using default`, error);
    return defaultValue;
  }
}

export async function setSetting<T>(key: string, value: T): Promise<void> {
  try {
    await withStoreTimeout(async () => {
      await store.set(key, value);
      await store.save();
    }, `setSetting(${key})`);
  } catch (error) {
    console.warn(`[ParseKit] setSetting(${key}) failed`, error);
  }
}