<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { t } from "../lib/i18n.svelte";
  import { pickInputFiles, pickInputFolder } from "../lib/picker";
  import { filterSupportedPaths, isSupportedFilePath } from "../lib/supportedExtensions";

  let {
    fileCount = null,
    disabled = false,
    onIngestStart,
    onIngestEnd,
    onFolder,
    onFiles,
  }: {
    fileCount: number | null;
    disabled?: boolean;
    onIngestStart?: () => void;
    onIngestEnd?: () => void;
    onFolder: (path: string, count: number, scanError?: string) => void;
    onFiles: (paths: string[]) => void;
  } = $props();

  let dragOver = $state(false);
  let scanning = $state(false);

  /** Normalize paths from drag-drop (may be file:// URLs on macOS). */
  function normalizePath(path: string): string {
    const trimmed = path.trim();
    if (!trimmed.startsWith("file://")) return trimmed;
    try {
      return decodeURIComponent(new URL(trimmed).pathname);
    } catch {
      return trimmed;
    }
  }

  async function ingestFolder(path: string) {
    const normalized = normalizePath(path);
    let count = 0;
    try {
      const scanned = await invoke<string[]>("scan_directory", { path: normalized });
      count = scanned.length;
    } catch (e) {
      onFolder(normalized, 0, e instanceof Error ? e.message : String(e));
      return;
    }
    onFolder(normalized, count);
  }

  async function ingestPaths(paths: string[]) {
    if (disabled || scanning) return;
    scanning = true;
    onIngestStart?.();
    const filePaths: string[] = [];

    try {
      for (const raw of paths) {
        const path = normalizePath(raw);
        const isDir = await invoke<boolean>("path_is_directory", { path });
        if (isDir) {
          await ingestFolder(path);
          return;
        }
        if (isSupportedFilePath(path)) {
          filePaths.push(path);
        }
      }

      const supported = filterSupportedPaths(filePaths);
      if (supported.length > 0) {
        onFiles(supported);
      } else if (paths.length > 0) {
        onFiles([]);
      }
    } finally {
      scanning = false;
      onIngestEnd?.();
    }
  }

  async function selectFiles() {
    if (disabled || scanning) return;
    const paths = await pickInputFiles();
    if (!paths?.length) return;
    await ingestPaths(paths);
  }

  async function selectFolder() {
    if (disabled || scanning) return;
    const path = await pickInputFolder();
    if (!path) return;
    scanning = true;
    onIngestStart?.();
    try {
      await ingestFolder(path);
    } finally {
      scanning = false;
      onIngestEnd?.();
    }
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onDragDropEvent((event) => {
        const { type } = event.payload;
        if (type === "enter" || type === "over") {
          dragOver = true;
        } else if (type === "leave") {
          dragOver = false;
        } else if (type === "drop") {
          dragOver = false;
          void ingestPaths(event.payload.paths);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  });
</script>

<div class="drop-zone" class:drag-over={dragOver} class:drop-zone-busy={scanning || disabled}>
  <div class="drop-zone-icon" aria-hidden="true">
    <svg width="40" height="40" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="8" y="6" width="24" height="28" rx="3" stroke="currentColor" stroke-width="1.5"/>
      <path d="M20 14v10M15 19h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>
  </div>
  <p class="drop-zone-title">{t("dropzone.title")}</p>
  <p class="drop-zone-hint">
    {scanning ? t("dropzone.scanning") : t("dropzone.hint")}
  </p>
  {#if fileCount !== null && fileCount > 0}
    <p class="drop-zone-ready">
      {fileCount === 1
        ? t("dropzone.filesReadyOne")
        : t("dropzone.filesReady", { count: fileCount })}
    </p>
  {/if}
  <div class="drop-zone-actions">
    <button type="button" disabled={disabled || scanning} onclick={selectFiles}>{t("dropzone.selectFiles")}</button>
    <button type="button" class="secondary" disabled={disabled || scanning} onclick={selectFolder}>{t("dropzone.selectFolder")}</button>
  </div>
</div>