<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { t } from "../lib/i18n";
  import { pickInputFiles, pickInputFolder } from "../lib/picker";

  const SUPPORTED_EXTENSIONS = new Set([
    "pdf", "doc", "docx", "docm", "odt", "rtf", "ppt", "pptx", "pptm", "odp",
    "xls", "xlsx", "xlsm", "ods", "csv", "tsv", "png", "jpg", "jpeg", "gif",
    "bmp", "tiff", "tif", "webp", "svg",
  ]);

  let {
    fileCount = null,
    onFolder,
    onFiles,
  }: {
    fileCount: number | null;
    onFolder: (path: string, count: number) => void;
    onFiles: (paths: string[]) => void;
  } = $props();

  let dragOver = $state(false);

  function isSupportedFile(path: string): boolean {
    const ext = path.split(".").pop()?.toLowerCase();
    return !!ext && SUPPORTED_EXTENSIONS.has(ext);
  }

  async function ingestPaths(paths: string[]) {
    const files: string[] = [];
    let folderPath: string | null = null;
    let folderCount = 0;

    for (const path of paths) {
      try {
        const scanned = await invoke<string[]>("scan_directory", { path });
        folderPath = path;
        folderCount = scanned.length;
        files.length = 0;
        break;
      } catch {
        if (isSupportedFile(path)) {
          files.push(path);
        }
      }
    }

    if (folderPath) {
      onFolder(folderPath, folderCount);
      return;
    }

    if (files.length > 0) {
      onFiles(files);
    }
  }

  async function selectFiles() {
    const paths = await pickInputFiles();
    if (!paths?.length) return;
    await ingestPaths(paths);
  }

  async function selectFolder() {
    const path = await pickInputFolder();
    if (!path) return;
    await ingestPaths([path]);
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

<div class="drop-zone" class:drag-over={dragOver}>
  <div class="drop-zone-icon" aria-hidden="true">
    <svg width="40" height="40" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="8" y="6" width="24" height="28" rx="3" stroke="currentColor" stroke-width="1.5"/>
      <path d="M20 14v10M15 19h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>
  </div>
  <p class="drop-zone-title">{t("dropzone.title")}</p>
  <p class="drop-zone-hint">{t("dropzone.hint")}</p>
  {#if fileCount !== null && fileCount > 0}
    <p class="drop-zone-ready">
      {fileCount === 1
        ? t("dropzone.filesReadyOne")
        : t("dropzone.filesReady", { count: fileCount })}
    </p>
  {/if}
  <div class="drop-zone-actions">
    <button type="button" onclick={selectFiles}>{t("dropzone.selectFiles")}</button>
    <button type="button" class="secondary" onclick={selectFolder}>{t("dropzone.selectFolder")}</button>
  </div>
</div>