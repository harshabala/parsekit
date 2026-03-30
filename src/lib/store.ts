import { LazyStore } from "@tauri-apps/plugin-store";

const store = new LazyStore("settings.json");

export async function getSetting<T>(key: string, defaultValue: T): Promise<T> {
  const val = await store.get<T>(key);
  return val !== undefined ? val : defaultValue;
}

export async function setSetting<T>(key: string, value: T): Promise<void> {
  await store.set(key, value);
  await store.save();
}
