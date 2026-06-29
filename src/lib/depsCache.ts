import { invoke } from "@tauri-apps/api/core";

export interface DepStatus {
  id: string;
  labelKey: string;
  installed: boolean;
  optional: boolean;
  brewHint: string;
}

let cached: DepStatus[] | null = null;
let warmStarted = false;

export function peekCachedDependencies(): DepStatus[] | null {
  return cached;
}

export function takeCachedDependencies(): DepStatus[] | null {
  const value = cached;
  cached = null;
  return value;
}

export function warmDependencies(): void {
  if (warmStarted) return;
  warmStarted = true;
  void invoke<DepStatus[]>("check_dependencies")
    .then((result) => {
      cached = result;
    })
    .catch(() => {
      warmStarted = false;
    });
}

export function resetDepsCacheForTests(): void {
  cached = null;
  warmStarted = false;
}

export function setCachedDependenciesForTests(value: DepStatus[]): void {
  cached = value;
}
