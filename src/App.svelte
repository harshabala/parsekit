<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getSetting, setSetting } from "./lib/store";
  import { runParse, type ParseEvent } from "./lib/sidecar";
  import type { OutputFormat, FileProgress, BatchResult, ThemeMode } from "./lib/types";
  import { MAX_RECENT_BATCHES } from "./lib/types";
  import {
    initLocale,
    locale,
    localeFromLegacyOcr,
    normalizeLocale,
    t,
    type AppLocale,
  } from "./lib/i18n";
  import { normalizeOcrLanguage, type OcrLanguageCode } from "./lib/ocrLanguages";
  import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";
  import DropZone from "./components/DropZone.svelte";
  import OutputFolderPicker from "./components/OutputFolderPicker.svelte";
  import FormatSelector from "./components/FormatSelector.svelte";
  import ProgressList from "./components/ProgressList.svelte";
  import RecentBatches from "./components/RecentBatches.svelte";
  import AboutScreen from "./components/AboutScreen.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import "./index.css";

  let inputDir = $state("");
  let selectedFiles = $state<string[]>([]);
  let outputDir = $state("");
  let format = $state<OutputFormat>("md");
  let isParsing = $state(false);
  let ocrEnabled = $state(true);
  let ocrLanguage = $state<OcrLanguageCode>("eng");
  let workers = $state(4);
  let files = $state<FileProgress[]>([]);
  let totalFiles = $state(0);
  let recentBatches = $state<BatchResult[]>([]);
  let showAbout = $state(false);
  let showSettings = $state(false);
  let theme = $state<ThemeMode>(DEFAULT_THEME);
  let inputFileCount = $state<number | null>(null);
  let errorMsg = $state<string | null>(null);

  let showProgress = $derived(isParsing || files.length > 0);
  let canRunParse = $derived(
    !isParsing &&
      !!outputDir &&
      inputFileCount !== null &&
      inputFileCount > 0
  );

  async function resolveDefaultWorkers(savedWorkers: number) {
    if (savedWorkers > 0) {
      workers = savedWorkers;
      return;
    }
    try {
      const sysInfo = await invoke<{ optimalWorkers: number }>("get_system_info");
      workers = sysInfo.optimalWorkers;
    } catch {
      workers = 4;
    }
  }

  async function handleThemeChange(mode: ThemeMode) {
    theme = mode;
    applyTheme(mode);
    await setSetting("theme", mode);
  }

  async function handleLocaleChange(code: AppLocale) {
    initLocale(code);
    await setSetting("locale", code);
  }

  async function handleOcrLanguageChange(code: OcrLanguageCode) {
    ocrLanguage = code;
    await setSetting("ocrLanguage", code);
  }

  function openSettings() {
    showAbout = false;
    showSettings = true;
  }

  function openAbout() {
    showSettings = false;
    showAbout = true;
  }

  onMount(async () => {
    theme = normalizeThemeMode(await getSetting("theme", DEFAULT_THEME));
    applyTheme(theme);

    outputDir = await getSetting("outputDir", "");
    if (!outputDir) {
      outputDir = await downloadDir();
      await setSetting("outputDir", outputDir);
    }

    format = await getSetting<OutputFormat>("format", "md");
    ocrEnabled = await getSetting("ocrEnabled", true);
    const savedLocale = await getSetting<AppLocale | null>("locale", null);
    const resolvedLocale = savedLocale
      ? normalizeLocale(savedLocale)
      : localeFromLegacyOcr(await getSetting("ocrLanguage", "eng"));
    initLocale(resolvedLocale);
    await setSetting("locale", resolvedLocale);
    ocrLanguage = normalizeOcrLanguage(await getSetting("ocrLanguage", "eng"));
    await setSetting("ocrLanguage", ocrLanguage);
    recentBatches = await getSetting<BatchResult[]>("recentBatches", []);
    await resolveDefaultWorkers(await getSetting<number>("workers", 0));

    const savedInput = await getSetting("inputDir", "");
    if (savedInput) {
      await handleFolderSelected(savedInput, null);
    }
  });

  function updateInputCount(count: number) {
    inputFileCount = count;
  }

  async function handleFolderSelected(path: string, count: number | null) {
    inputDir = path;
    selectedFiles = [];
    await setSetting("inputDir", path);
    if (count !== null) {
      updateInputCount(count);
      return;
    }
    try {
      const scanned = await invoke<string[]>("scan_directory", { path });
      updateInputCount(scanned.length);
    } catch {
      updateInputCount(0);
    }
  }

  async function handleFilesSelected(paths: string[]) {
    selectedFiles = paths;
    inputDir = "";
    await setSetting("inputDir", "");
    updateInputCount(paths.length);
  }

  async function handleOutputSelect(path: string) {
    outputDir = path;
    await setSetting("outputDir", outputDir);
  }

  async function handleFormatChange(f: OutputFormat) {
    format = f;
    await setSetting("format", format);
  }

  async function handleOcrEnabledChange() {
    await setSetting("ocrEnabled", ocrEnabled);
  }

  async function startParse() {
    if (!outputDir || inputFileCount === 0) return;

    errorMsg = null;

    let filesToParse: string[];
    try {
      if (selectedFiles.length > 0) {
        filesToParse = selectedFiles;
      } else if (inputDir) {
        filesToParse = await invoke<string[]>("scan_directory", { path: inputDir });
      } else {
        errorMsg = t("errors.addFiles");
        return;
      }
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      return;
    }

    if (filesToParse.length === 0) {
      errorMsg = t("errors.noSupported");
      return;
    }

    try {
      await invoke("trigger_haptic");
    } catch {}

    isParsing = true;
    totalFiles = filesToParse.length;
    files = filesToParse.map((path) => ({
      name: path.split(/[/\\]/).pop() || path,
      status: "pending" as const,
    }));

    try {
      await runParse(
        {
          inputDir: inputDir || filesToParse[0],
          files: filesToParse,
          outputDir,
          format,
          ocrEnabled,
          ocrLanguage,
          workers,
        },
        (event: ParseEvent) => {
          if (event.type === "start") {
            totalFiles = event.total || 0;
          } else if (event.type === "progress") {
            const name = event.file || "";
            let status: FileProgress["status"] = "pending";
            if (event.status === "completed") status = "done";
            else if (event.status === "parsing") status = "parsing";
            else if (event.status === "error") status = "error";
            else if (event.status === "skipped") status = "skipped";

            const existingIndex = files.findIndex((f) => f.name === name);
            if (existingIndex !== -1) {
              files[existingIndex] = {
                ...files[existingIndex],
                status,
                path: event.path || files[existingIndex].path,
                error: event.error,
              };
            } else {
              files = [
                { name, status, path: event.path, error: event.error },
                ...files,
              ];
            }
          } else if (event.type === "done") {
            isParsing = false;
            totalFiles = totalFiles || files.length;
            addToHistory();
          } else if (event.type === "error") {
            isParsing = false;
            errorMsg = event.message || t("errors.parseFailed");
            console.error(event.message);
          }
        }
      );
    } catch (e) {
      isParsing = false;
      errorMsg = e instanceof Error ? e.message : String(e);
      console.error(e);
    }
  }

  async function addToHistory() {
    const parsed = files.filter((f) => f.status === "done").length;
    const errors = files.filter((f) => f.status === "error").length;
    const newBatch: BatchResult = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      inputDir: inputDir || "selected files",
      outputDir,
      format,
      fileCount: files.length,
      parsed,
      errors,
    };
    recentBatches = [newBatch, ...recentBatches.slice(0, MAX_RECENT_BATCHES - 1)];
    await setSetting("recentBatches", recentBatches);
  }

  async function copyToClipboard() {
    const lastFile = files.find((f) => f.status === "done");
    if (lastFile?.path) {
      try {
        const bytes = await invoke<number[]>("copy_file_to_clipboard", {
          path: lastFile.path,
        });
        const content = new TextDecoder().decode(new Uint8Array(bytes));
        await writeText(content);
        try {
          await invoke("trigger_haptic");
        } catch {}
      } catch {
        await writeText(lastFile.path);
      }
    }
  }

  async function openFolder(path: string) {
    try {
      await invoke("open_in_finder", { path });
    } catch {
      const { Command } = await import("@tauri-apps/plugin-shell");
      await Command.create("open", [path]).spawn();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "r") {
      e.preventDefault();
      if (canRunParse) {
        startParse();
      }
    }
    if (e.key === "Escape") {
      if (showSettings) showSettings = false;
      else if (showAbout) showAbout = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div id="app">
  <header>
    <span>{t("app.name")}</span>
    <div class="header-actions">
      <button
        class="icon-btn"
        onclick={openSettings}
        title={t("header.settings")}
        aria-label={t("header.settings")}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"
            stroke="currentColor"
            stroke-width="1.5"
          />
          <path
            d="M8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.34 3.34l1.13 1.13M11.53 11.53l1.13 1.13M3.34 12.66l1.13-1.13M11.53 4.47l1.13-1.13"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
          />
        </svg>
      </button>
      <button
        class="icon-btn"
        onclick={openAbout}
        title={t("header.about")}
        aria-label={t("header.about")}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5"/>
          <path
            d="M7 6.5C7 5.67 7.67 5 8.5 5C9.33 5 10 5.67 10 6.5C10 7.17 9.5 7.5 9 7.8C8.7 7.97 8.5 8.17 8.5 8.5V9"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
          <circle cx="8.5" cy="11" r="0.75" fill="currentColor"/>
        </svg>
      </button>
    </div>
  </header>

  <main>
    <div class="section">
      <div class="section-title">{t("config.title")}</div>
      <div class="card">
        <OutputFolderPicker value={outputDir} onSelect={handleOutputSelect} />

        <div class="row">
          <span>{t("config.format")}</span>
          <FormatSelector value={format} onChange={handleFormatChange} />
        </div>
        {#if format !== "json"}
          <div class="file-count-preview caption-hint">{t("config.spreadsheetJsonHint")}</div>
        {/if}

        <div class="row ocr-row">
          <div class="ocr-toggle">
            <input
              type="checkbox"
              bind:checked={ocrEnabled}
              id="ocr-toggle"
              onchange={handleOcrEnabledChange}
            />
            <label for="ocr-toggle">{t("config.ocr")}</label>
          </div>
        </div>
      </div>
    </div>

    <DropZone
      fileCount={inputFileCount}
      onFolder={handleFolderSelected}
      onFiles={handleFilesSelected}
    />

    {#if showProgress}
      <ProgressList {files} total={totalFiles || files.length} {isParsing} />
    {/if}

    <div class="section run-section">
      <button
        class="run-parse-btn"
        disabled={!canRunParse}
        onclick={startParse}
      >
        {isParsing ? t("run.parsing") : t("run.runParse")}
      </button>
      {#if errorMsg}
        <div class="error-banner" role="alert">{errorMsg}</div>
      {/if}
      {#if !isParsing && files.length > 0 && files.some((f) => f.status === "done")}
        <div class="row" style="margin-top: 8px;">
          <button class="secondary" style="flex: 1" onclick={() => openFolder(outputDir)}>
            {t("run.openOutput")}
          </button>
          <button class="secondary" style="flex: 1" onclick={copyToClipboard}>
            {t("run.copyLast")}
          </button>
        </div>
      {/if}
    </div>

    {#if !isParsing}
      <RecentBatches batches={recentBatches} onOpenFolder={openFolder} />
    {/if}
  </main>

  {#if showSettings}
    <SettingsScreen
      locale={locale}
      {ocrLanguage}
      {ocrEnabled}
      {theme}
      onLocaleChange={handleLocaleChange}
      onOcrLanguageChange={handleOcrLanguageChange}
      onThemeChange={handleThemeChange}
      onClose={() => (showSettings = false)}
    />
  {/if}

  {#if showAbout}
    <AboutScreen onClose={() => (showAbout = false)} />
  {/if}
</div>