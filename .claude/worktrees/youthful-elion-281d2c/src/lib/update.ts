import { invoke } from "@tauri-apps/api/core";

export interface UpdateInfo {
  available: boolean;
  version: string | null;
  body: string | null;
  download_url: string | null;
}

/** Set `VITE_MOCK_UPDATE=1` when running dev to preview the update banner UI. */
const MOCK_UPDATE_IN_DEV =
  import.meta.env.DEV && import.meta.env.VITE_MOCK_UPDATE === "1";

export async function checkForUpdate(): Promise<UpdateInfo> {
  if (MOCK_UPDATE_IN_DEV) {
    return {
      available: true,
      version: "0.2.99",
      body: "Mock release notes (dev preview).",
      download_url: null,
    };
  }
  return invoke<UpdateInfo>("check_for_update");
}

export async function installUpdate(): Promise<void> {
  if (MOCK_UPDATE_IN_DEV) {
    throw new Error("Mock update install — set VITE_MOCK_UPDATE=0 to test a real install.");
  }
  await invoke("install_update");
}