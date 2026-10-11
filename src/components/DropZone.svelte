<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import FilePlusIcon from "phosphor-svelte/lib/FilePlusIcon";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
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

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

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
    <FilePlusIcon size={32} weight="regular" />
  </div>
  <p class="drop-zone-title">{t("dropzone.title")}</p>
  {#key scanning}
    <p class="drop-zone-hint" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
      {scanning ? t("dropzone.scanning") : t("dropzone.hint")}
    </p>
  {/key}
  {#if fileCount !== null && fileCount > 0}
    <p class="drop-zone-ready" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
      {fileCount === 1
        ? t("dropzone.filesReadyOne")
        : t("dropzone.filesReady", { count: fileCount })}
    </p>
  {/if}
  <div class="drop-zone-actions">
    <button
      type="button"
      class="drop-zone-btn-primary"
      disabled={disabled || scanning}
      onclick={selectFiles}>{t("dropzone.selectFiles")}</button
    >
    <button
      type="button"
      class="secondary drop-zone-btn-secondary"
      disabled={disabled || scanning}
      onclick={selectFolder}>{t("dropzone.selectFolder")}</button
    >
  </div>
</div>