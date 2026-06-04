<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";

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
  import {
    applyParseProgressEvent,
    settleInFlightOnAbort,
  } from "./lib/progress";
  import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";
  import DropZone from "./components/DropZone.svelte";
  import OutputFolderPicker from "./components/OutputFolderPicker.svelte";
  import FormatSelector from "./components/FormatSelector.svelte";
  import ProgressList from "./components/ProgressList.svelte";
  import RecentBatches from "./components/RecentBatches.svelte";
  import HistoryScreen from "./components/HistoryScreen.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import {
    bannerFlyIn,
    bannerFlyOut,
    buttonFadeIn,
    buttonFadeOut,
    panelFadeIn,
    panelFadeOut,
    panelFlyIn,
    panelFlyOut,
    sectionFlyIn,
    sectionFlyOut,
  } from "./lib/motion";
  import "./index.css";

  const reducedMotion = $derived(prefersReducedMotion.current);
  const mainFlyIn = $derived(panelFlyIn(reducedMotion));
  const mainFlyOut = $derived(panelFlyOut(reducedMotion));
  const mainFadeIn = $derived(panelFadeIn(reducedMotion));
  const mainFadeOut = $derived(panelFadeOut(reducedMotion));
  const bannerFlyInParams = $derived(bannerFlyIn(reducedMotion));
  const bannerFlyOutParams = $derived(bannerFlyOut(reducedMotion));
  const sectionFlyInParams = $derived(sectionFlyIn(reducedMotion));
  const sectionFlyOutParams = $derived(sectionFlyOut(reducedMotion));
  const buttonFadeInParams = $derived(buttonFadeIn(reducedMotion));
  const buttonFadeOutParams = $derived(buttonFadeOut(reducedMotion));

  let inputDir = $state("");
  let selectedFiles = $state<string[]>([]);
  let outputDir = $state("");
  let format = $state<OutputFormat>("md");
  let isParsing = $state(false);
  let ocrEnabled = $state(true);
  let ocrLanguage = $state<OcrLanguageCode>("eng");
  let workers = $state(4);
  let files = $state<FileProgress[]>([]);
  let lastParsingId = $state<string | null>(null);
  let totalFiles = $state(0);
  let recentBatches = $state<BatchResult[]>([]);
  let showSettings = $state(false);
  let showHistory = $state(false);
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
    showHistory = false;
    showSettings = true;
  }

  function openHistory() {
    showSettings = false;
    showHistory = true;
  }

  let latestBatch = $derived(recentBatches[0] ?? null);

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
      await handleFolderSelected(savedInput, null, { silent: true });
    }
  });

  function updateInputCount(count: number) {
    inputFileCount = count;
  }

  async function handleFolderSelected(
    path: string,
    count: number | null,
    options?: { silent?: boolean }
  ) {
    inputDir = path;
    selectedFiles = [];
    await setSetting("inputDir", path);

    let resolved = count;
    if (count === null) {
      try {
        const scanned = await invoke<string[]>("scan_directory", { path });
        resolved = scanned.length;
      } catch (e) {
        updateInputCount(0);
        if (!options?.silent) {
          errorMsg = e instanceof Error ? e.message : String(e);
        }
        return;
      }
    }

    updateInputCount(resolved ?? 0);
    if ((resolved ?? 0) === 0 && !options?.silent) {
      errorMsg = t("errors.noSupported");
      noticeMsg = null;
    } else if ((resolved ?? 0) > 0) {
      errorMsg = null;
    }
  }

  function handleFolderFromDropZone(path: string, count: number, scanError?: string) {
    if (scanError) {
      inputDir = path;
      selectedFiles = [];
      void setSetting("inputDir", path);
      updateInputCount(0);
      errorMsg = scanError;
      noticeMsg = null;
      return;
    }
    void handleFolderSelected(path, count);
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

  function stopParseUi(notice: string, error: string | null = null) {
    isParsing = false;
    files = settleInFlightOnAbort(files, t("errors.batchInterrupted"));
    lastParsingId = null;
    errorMsg = error;
    noticeMsg = notice;
  }

  function cancelParse() {
    parseRun?.cancel();
    parseRun = null;
    stopParseUi(t("errors.parseCancelled"));
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
    lastParsingId = null;
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
            const applied = applyParseProgressEvent(
              files,
              {
                type: "progress",
                file: event.file,
                sourcePath: event.sourcePath,
                status: event.status,
                path: event.path,
                error: event.error,
              },
              lastParsingId
            );
            files = applied.files;
            lastParsingId = applied.lastParsingId;
          } else if (event.type === "done") {
            isParsing = false;
            lastParsingId = null;
            totalFiles = totalFiles || files.length;
            void addToHistory();
            const parsed = files.filter((f) => f.status === "done").length;
            const errCount = files.filter((f) => f.status === "error").length;
            void invoke("show_completion_notification", {
              title: t("app.name"),
              body: t("run.notifyDone", { parsed, errors: errCount }),
            }).catch(() => {});
          } else if (event.type === "error") {
            parseRun = null;
            stopParseUi(
              t("errors.parseFailed"),
              event.message || t("errors.parseFailed")
            );
            console.error(event.message);
          }
        }
    );

    try {
      await parseRun.promise;
    } catch (e) {
      if (isParsing) {
        stopParseUi(
          t("errors.parseFailed"),
          e instanceof Error ? e.message : String(e)
        );
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
      else if (showHistory) showHistory = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="shell">
  {#if !showSettings && !showHistory}
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
          <div
            class="file-count-preview caption-hint"
            in:fade={mainFadeIn}
            out:fade={mainFadeOut}
          >
            {t("config.spreadsheetJsonHint")}
          </div>
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
      onFolder={handleFolderFromDropZone}
      onFiles={handleFilesSelected}
    />

    {#if showProgress}
      <div in:fly={sectionFlyInParams} out:fly={sectionFlyOutParams}>
        <ProgressList
          {files}
          total={totalFiles || files.length}
          {isParsing}
          {lastParsingId}
        />
      </div>
    {/if}

    <div class="section run-section">
      {#key isParsing}
        <div in:fade={buttonFadeInParams} out:fade={buttonFadeOutParams}>
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
        </div>
      {/key}
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
        <div class="post-parse-actions" in:fly={sectionFlyInParams} out:fly={sectionFlyOutParams}>
          <button
            type="button"
            class="secondary post-parse-open-btn"
            onclick={() => openFolder(outputDir)}
          >
            {t("run.openOutput")}
          </button>
        </div>
      {/if}
    </div>

    {#if !isParsing}
      <div in:fade={mainFadeIn} out:fade={mainFadeOut}>
        <RecentBatches
          {latestBatch}
          showHistoryButton={recentBatches.length > 0}
          onOpenFolder={openFolder}
          onOpenHistory={openHistory}
        />
      </div>
    {/if}
  </main>
        </div>
      </div>
    {/key}
  {/if}

  {#if showHistory}
    {#key "history"}
      <div class="motion-panel" in:fly={mainFlyIn} out:fly={mainFlyOut}>
        <div class="motion-panel-content" in:fade={mainFadeIn} out:fade={mainFadeOut}>
          <HistoryScreen
            batches={recentBatches}
            onOpenFolder={openFolder}
            onClose={() => (showHistory = false)}
          />
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