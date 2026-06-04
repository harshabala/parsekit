<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getSetting, setSetting } from "./lib/store";
  import { runParse, type ParseEvent, type ParseRunHandle } from "./lib/sidecar";
  import type { OutputFormat, FileProgress, BatchResult, ThemeMode } from "./lib/types";
  import { MAX_RECENT_BATCHES } from "./lib/types";
  import {
    initLocale,
    getLocale,
    localeFromLegacyOcr,
    normalizeLocale,
    t,
    type AppLocale,
  } from "./lib/i18n.svelte";
  import {
    isKnownOcrLanguage,
    normalizeOcrLanguage,
    type OcrLanguageCode,
  } from "./lib/ocrLanguages";
  import { fileBaseName, filterSupportedPaths } from "./lib/supportedExtensions";
  import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";
  import DropZone from "./components/DropZone.svelte";
  import OutputFolderPicker from "./components/OutputFolderPicker.svelte";
  import FormatSelector from "./components/FormatSelector.svelte";
  import ProgressList from "./components/ProgressList.svelte";
  import RecentBatches from "./components/RecentBatches.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import {
    bannerFlyIn,
    bannerFlyOut,
    panelFadeIn,
    panelFadeOut,
    panelFlyIn,
    panelFlyOut,
  } from "./lib/motion";
  import "./index.css";

  const reducedMotion = $derived(prefersReducedMotion.current);
  const mainFlyIn = $derived(panelFlyIn(reducedMotion));
  const mainFlyOut = $derived(panelFlyOut(reducedMotion));
  const mainFadeIn = $derived(panelFadeIn(reducedMotion));
  const mainFadeOut = $derived(panelFadeOut(reducedMotion));
  const bannerFlyInParams = $derived(bannerFlyIn(reducedMotion));
  const bannerFlyOutParams = $derived(bannerFlyOut(reducedMotion));

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
  let showSettings = $state(false);
  let theme = $state<ThemeMode>(DEFAULT_THEME);
  let inputFileCount = $state<number | null>(null);
  let errorMsg = $state<string | null>(null);
  let noticeMsg = $state<string | null>(null);
  let parseRun = $state<ParseRunHandle | null>(null);
  let isIngesting = $state(false);
  let launchAtLogin = $state(false);

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

  async function syncTrayMenu() {
    try {
      await invoke("update_tray_menu_labels", {
        openLabel: t("tray.open"),
        quitLabel: t("tray.quit"),
      });
    } catch {
      /* tray may not exist in web-only dev */
    }
  }

  async function handleLocaleChange(code: AppLocale) {
    initLocale(code);
    await setSetting("locale", code);
    await syncTrayMenu();
  }

  async function handleOcrLanguageChange(code: OcrLanguageCode) {
    ocrLanguage = code;
    await setSetting("ocrLanguage", code);
  }

  function openSettings() {
    showSettings = true;
  }

  async function quitApp() {
    try {
      await invoke("quit_app");
    } catch {
      /* web-only dev */
    }
  }

  onMount(async () => {
    theme = normalizeThemeMode(await getSetting("theme", DEFAULT_THEME));
    applyTheme(theme);

    const savedLocale = await getSetting<AppLocale | null>("locale", null);
    const resolvedLocale = savedLocale
      ? normalizeLocale(savedLocale)
      : localeFromLegacyOcr(await getSetting("ocrLanguage", "eng"));
    initLocale(resolvedLocale);
    await setSetting("locale", resolvedLocale);

    outputDir = await getSetting("outputDir", "");
    if (!outputDir) {
      outputDir = await downloadDir();
      await setSetting("outputDir", outputDir);
    }

    format = await getSetting<OutputFormat>("format", "md");
    ocrEnabled = await getSetting("ocrEnabled", true);
    const rawOcr = String(await getSetting("ocrLanguage", "eng"));
    if (!isKnownOcrLanguage(rawOcr)) {
      noticeMsg = t("settings.ocrMigrated");
    }
    ocrLanguage = normalizeOcrLanguage(rawOcr);
    await setSetting("ocrLanguage", ocrLanguage);
    recentBatches = await getSetting<BatchResult[]>("recentBatches", []);
    await resolveDefaultWorkers(await getSetting<number>("workers", 0));
    launchAtLogin = await getSetting<boolean>("launchAtLogin", false);
    if (launchAtLogin) {
      try {
        await invoke("set_launch_at_login", { enabled: true });
      } catch {
        launchAtLogin = false;
        await setSetting("launchAtLogin", false);
      }
    }
    await syncTrayMenu();

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
    const supported = filterSupportedPaths(paths);
    selectedFiles = supported;
    inputDir = "";
    await setSetting("inputDir", "");
    updateInputCount(supported.length);
    if (paths.length > 0 && supported.length === 0) {
      errorMsg = t("errors.noSupported");
    } else {
      errorMsg = null;
    }
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

  async function handleWorkersChange(value: number) {
    workers = value;
    await setSetting("workers", value);
  }

  async function handleLaunchAtLoginChange(enabled: boolean) {
    launchAtLogin = enabled;
    await invoke("set_launch_at_login", { enabled });
    await setSetting("launchAtLogin", enabled);
  }

  function cancelParse() {
    parseRun?.cancel();
    parseRun = null;
    isParsing = false;
    errorMsg = null;
    noticeMsg = t("errors.parseCancelled");
  }

  async function copyOutputPath() {
    if (!outputDir) return;
    await writeText(outputDir);
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
      id: path,
      name: fileBaseName(path),
      status: "pending" as const,
    }));

    parseRun = runParse(
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
            const sourcePath = event.sourcePath;
            const displayName = event.file || (sourcePath ? fileBaseName(sourcePath) : "");
            let status: FileProgress["status"] = "pending";
            if (event.status === "completed") status = "done";
            else if (event.status === "parsing") status = "parsing";
            else if (event.status === "error") status = "error";
            else if (event.status === "skipped") status = "skipped";

            const existingIndex = sourcePath
              ? files.findIndex((f) => f.id === sourcePath)
              : files.findIndex(
                  (f) => f.name === displayName && f.status !== "done" && f.status !== "error"
                );
            if (existingIndex !== -1) {
              files[existingIndex] = {
                ...files[existingIndex],
                status,
                outputPath: event.path || files[existingIndex].outputPath,
                error: event.error,
              };
            } else if (sourcePath) {
              files = [
                {
                  id: sourcePath,
                  name: displayName,
                  status,
                  outputPath: event.path,
                  error: event.error,
                },
                ...files,
              ];
            }
          } else if (event.type === "done") {
            isParsing = false;
            totalFiles = totalFiles || files.length;
            void addToHistory();
            const parsed = files.filter((f) => f.status === "done").length;
            const errCount = files.filter((f) => f.status === "error").length;
            void invoke("show_completion_notification", {
              title: t("app.name"),
              body: t("run.notifyDone", { parsed, errors: errCount }),
            }).catch(() => {});
          } else if (event.type === "error") {
            isParsing = false;
            errorMsg = event.message || t("errors.parseFailed");
            console.error(event.message);
          }
        }
    );

    try {
      await parseRun.promise;
    } catch (e) {
      if (isParsing) {
        isParsing = false;
        errorMsg = e instanceof Error ? e.message : String(e);
        console.error(e);
      }
    } finally {
      parseRun = null;
    }
  }

  async function addToHistory() {
    const parsed = files.filter((f) => f.status === "done").length;
    const errors = files.filter((f) => f.status === "error").length;
    const newBatch: BatchResult = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      inputDir: inputDir || t("recent.selectedFiles"),
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
    if (lastFile?.outputPath) {
      try {
        const bytes = await invoke<number[]>("copy_file_to_clipboard", {
          path: lastFile.outputPath,
        });
        const content = new TextDecoder().decode(new Uint8Array(bytes));
        await writeText(content);
        try {
          await invoke("trigger_haptic");
        } catch {}
      } catch {
        await writeText(lastFile.outputPath);
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
    if (e.key === "Escape" && showSettings) {
      showSettings = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="shell">
  {#if !showSettings}
    {#key "main"}
      <div class="motion-panel" in:fly={mainFlyIn} out:fly={mainFlyOut}>
        <div class="motion-panel-content" in:fade={mainFadeIn} out:fade={mainFadeOut}>
  <header>
    <span>{t("app.name")}</span>
    <div class="header-actions">
      <button
        class="icon-btn"
        onclick={openSettings}
        title={t("header.settings")}
        aria-label={t("header.settings")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z"
            stroke="currentColor"
            stroke-width="2"
          />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <button
        class="icon-btn"
        onclick={quitApp}
        title={t("header.quit")}
        aria-label={t("header.quit")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2" />
          <path
            d="M15 9 9 15M9 9l6 6"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
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
      disabled={isIngesting || isParsing}
      onIngestStart={() => (isIngesting = true)}
      onIngestEnd={() => (isIngesting = false)}
      onFolder={handleFolderSelected}
      onFiles={handleFilesSelected}
    />

    {#if showProgress}
      <ProgressList {files} total={totalFiles || files.length} {isParsing} />
    {/if}

    <div class="section run-section">
      {#if isParsing}
        <button type="button" class="secondary run-parse-btn" onclick={cancelParse}>
          {t("run.cancel")}
        </button>
      {:else}
        <button
          type="button"
          class="run-parse-btn"
          disabled={!canRunParse}
          onclick={startParse}
        >
          {t("run.runParse")}
        </button>
      {/if}
      {#if noticeMsg}
        <div
          class="notice-banner"
          role="status"
          in:fly={bannerFlyInParams}
          out:fly={bannerFlyOutParams}
        >
          {noticeMsg}
        </div>
      {/if}
      {#if errorMsg}
        <div
          class="error-banner"
          role="alert"
          in:fly={bannerFlyInParams}
          out:fly={bannerFlyOutParams}
        >
          {errorMsg}
        </div>
      {/if}
      {#if !isParsing && files.length > 0 && files.some((f) => f.status === "done")}
        <div class="row" style="margin-top: 8px;">
          <button type="button" class="secondary" style="flex: 1" onclick={() => openFolder(outputDir)}>
            {t("run.openOutput")}
          </button>
          <button type="button" class="secondary" style="flex: 1" onclick={copyOutputPath}>
            {t("run.copyOutputPath")}
          </button>
        </div>
        <div class="row" style="margin-top: 8px;">
          <button type="button" class="secondary" style="flex: 1" onclick={copyToClipboard}>
            {t("run.copyLast")}
          </button>
        </div>
      {/if}
    </div>

    {#if !isParsing}
      <RecentBatches batches={recentBatches} onOpenFolder={openFolder} />
    {/if}
  </main>
        </div>
      </div>
    {/key}
  {/if}

  {#if showSettings}
    {#key "settings"}
      <div class="motion-panel" in:fly={mainFlyIn} out:fly={mainFlyOut}>
        <div class="motion-panel-content" in:fade={mainFadeIn} out:fade={mainFadeOut}>
    <SettingsScreen
      locale={getLocale()}
      {ocrLanguage}
      {ocrEnabled}
      {theme}
      {workers}
      {launchAtLogin}
      onLocaleChange={handleLocaleChange}
      onOcrLanguageChange={handleOcrLanguageChange}
      onThemeChange={handleThemeChange}
      onWorkersChange={handleWorkersChange}
      onLaunchAtLoginChange={handleLaunchAtLoginChange}
      onClose={() => (showSettings = false)}
    />
        </div>
      </div>
    {/key}
  {/if}
</div>