import { invoke } from "@tauri-apps/api/core";

/** Native file picker (no sheet parent — required for menu-bar popover on macOS). */
export async function pickInputFiles(): Promise<string[] | null> {
  return invoke<string[] | null>("pick_input_files");
}

export async function pickInputFolder(): Promise<string | null> {
  return invoke<string | null>("pick_input_folder");
}

export async function pickOutputFolder(): Promise<string | null> {
  return invoke<string | null>("pick_output_folder");
}