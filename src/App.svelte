<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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
  import AboutScreen from "./components/AboutScreen.svelte";
  import OnboardingChecklist from "./components/OnboardingChecklist.svelte";
  import UpdateBanner from "./components/UpdateBanner.svelte";
  import { checkForUpdate, installUpdate, type UpdateInfo } from "./lib/update";
  import { pickOutputFolder } from "./lib/picker";
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
  let showAbout = $state(false);
  let showHistory = $state(false);
  let theme = $state<ThemeMode>(DEFAULT_THEME);
  let inputFileCount = $state<number | null>(null);
  let errorMsg = $state<string | null>(null);
  let noticeMsg = $state<string | null>(null);
  let parseRun = $state<ParseRunHandle | null>(null);
  let isIngesting = $state(false);
  let launchAtLogin = $state(false);
  let finderActionInstalled = $state(false);
  let finderActionBusy = $state(false);
  let finderActionNotice = $state<string | null>(null);
  let showOnboarding = $state(false);
  let showInstallHint = $state(false);
  let configCollapsed = $state(false);
  let hasSuccessfulParse = $state(false);
  let appVersion = $state("0.2.0");
  let updateAvailable = $state<UpdateInfo | null>(null);
  let isInstallingUpdate = $state(false);
  let updateError = $state<string | null>(null);
  let updateCheckBusy = $state(false);
  let updateStatusNote = $state<string | null>(null);
  let updateStatusOk = $state(false);

  const PARSE_STALL_MS = 90_000;
  let lastParseEventAt = 0;
  let parseStallTimer: ReturnType<typeof setInterval> | null = null;

  let showProgress = $derived(isParsing || files.length > 0);
  let canRunParse = $derived(
    !isParsing &&
      !!outputDir &&
      inputFileCount !== null &&
      inputFileCount > 0
  );
  let outputDirConfigured = $derived(!!outputDir);
  let filesReady = $derived((inputFileCount ?? 0) > 0);

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
    showAbout = false;
    showSettings = true;
    void refreshFinderActionStatus();
  }

  async function refreshFinderActionStatus() {
    try {
      finderActionInstalled = await invoke<boolean>("finder_quick_action_installed");
    } catch {
      finderActionInstalled = false;
    }
  }

  async function installFinderQuickAction() {
    finderActionBusy = true;
    finderActionNotice = null;
    try {
      const msg = await invoke<string>("install_finder_quick_action");
      finderActionNotice = msg || t("settings.finderInstalled");
      await refreshFinderActionStatus();
    } catch (e) {
      finderActionNotice =
        (e instanceof Error ? e.message : String(e)) || t("settings.finderInstallFailed");
    } finally {
      finderActionBusy = false;
    }
  }

  async function ingestExternalPaths(paths: string[]) {
    const supported = filterSupportedPaths(paths);
    if (supported.length === 0) {
      errorMsg = t("errors.noSupported");
      return;
    }
    selectedFiles = supported;
    inputDir = "";
    await setSetting("inputDir", "");
    updateInputCount(supported.length);
    errorMsg = null;
    noticeMsg = null;
    showSettings = false;
    showAbout = false;
    showHistory = false;
    try {
      await invoke("trigger_haptic");
    } catch {}
    void openPopoverFromExternal();
  }

  async function openPopoverFromExternal() {
    try {
      await invoke("show_main_window");
    } catch {
      /* dev / web */
    }
  }

  function openHistory() {
    showSettings = false;
    showHistory = true;
  }

  async function rerunBatch(batch: BatchResult) {
    showHistory = false;
    showSettings = false;
    showAbout = false;
    outputDir = batch.outputDir;
    format = batch.format;
    await setSetting("outputDir", outputDir);
    await setSetting("format", format);

    if (batch.sourcePaths && batch.sourcePaths.length > 0) {
      await ingestExternalPaths(batch.sourcePaths);
      return;
    }

    const selectedLabel = t("recent.selectedFiles");
    if (batch.inputDir && batch.inputDir !== selectedLabel) {
      await handleFolderSelected(batch.inputDir, null);
      return;
    }

    noticeMsg = t("errors.addFiles");
  }

  let latestBatch = $derived(recentBatches[0] ?? null);

  async function quitApp() {
    try {
      await invoke("quit_app");
    } catch {
      /* web-only dev */
    }
  }

  function scheduleBackgroundUpdateCheck() {
    void checkForUpdate()
      .then((info) => {
        if (info.available) {
          updateAvailable = info;
        }
      })
      .catch(() => {
        /* silent — offline or misconfigured endpoint */
      });
  }

  async function handleCheckForUpdates() {
    updateStatusNote = null;
    updateStatusOk = false;
    updateError = null;
    updateCheckBusy = true;
    try {
      const info = await checkForUpdate();
      if (info.available) {
        updateAvailable = info;
        showSettings = false;
        showAbout = false;
        updateStatusNote = null;
      } else {
        updateStatusNote = t("update.upToDate", { version: appVersion });
        updateStatusOk = true;
      }
    } catch {
      updateStatusNote = t("update.checkFailed");
    } finally {
      updateCheckBusy = false;
    }
  }

  async function installAvailableUpdate() {
    isInstallingUpdate = true;
    updateError = null;
    try {
      await installUpdate();
    } catch (e) {
      updateError =
        e instanceof Error ? e.message : String(e) || t("update.installFailed");
      isInstallingUpdate = false;
    }
  }

  function dismissUpdateBanner() {
    updateAvailable = null;
    updateError = null;
  }

  onMount(() => {
    let unlistenOpen: (() => void) | undefined;

    void (async () => {
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
    hasSuccessfulParse = await getSetting("hasSuccessfulParse", false);
    configCollapsed = hasSuccessfulParse;
    const onboardingDone = await getSetting("hasCompletedOnboarding", false);
    if (!onboardingDone) {
      showOnboarding = true;
      try {
        showInstallHint = !(await invoke<boolean>("is_installed_in_applications"));
      } catch {
        showInstallHint = false;
      }
      void openPopoverFromExternal();
    }
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

    try {
      const info = await invoke<{ version?: string }>("get_system_info");
      if (info.version) appVersion = info.version;
    } catch {
      /* keep default */
    }

    scheduleBackgroundUpdateCheck();

    const savedInput = await getSetting("inputDir", "");
    if (savedInput) {
      await handleFolderSelected(savedInput, null, { silent: true });
    }

    unlistenOpen = await listen<string[]>("open-files", (event) => {
      const paths = event.payload;
      if (paths?.length) {
        void ingestExternalPaths(paths);
      }
    });
    })();

    return () => {
      unlistenOpen?.();
    };
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

  async function dismissOnboarding() {
    showOnboarding = false;
    await setSetting("hasCompletedOnboarding", true);
  }

  async function onboardingPickOutput() {
    const selected = await pickOutputFolder();
    if (selected) {
      await handleOutputSelect(selected);
    }
  }

  function toggleConfigCollapsed() {
    configCollapsed = !configCollapsed;
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

  function clearParseStallWatchdog() {
    if (parseStallTimer) {
      clearInterval(parseStallTimer);
      parseStallTimer = null;
    }
  }

  function touchParseActivity() {
    lastParseEventAt = Date.now();
  }

  function startParseStallWatchdog() {
    clearParseStallWatchdog();
    touchParseActivity();
    parseStallTimer = setInterval(() => {
      if (!isParsing) return;
      if (Date.now() - lastParseEventAt < PARSE_STALL_MS) return;
      parseRun?.cancel();
      parseRun = null;
      stopParseUi(t("errors.engineStalled"), t("errors.engineStalled"));
    }, 5000);
  }

  function stopParseUi(notice: string, error: string | null = null) {
    clearParseStallWatchdog();
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
    startParseStallWatchdog();
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
          touchParseActivity();
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
            clearParseStallWatchdog();
            isParsing = false;
            lastParsingId = null;
            totalFiles = totalFiles || files.length;
            void addToHistory();
            if (!hasSuccessfulParse) {
              hasSuccessfulParse = true;
              configCollapsed = true;
              void setSetting("hasSuccessfulParse", true);
            }
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
      clearParseStallWatchdog();
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
      sourcePaths:
        selectedFiles.length > 0 ? [...selectedFiles] : undefined,
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
      if (showAbout) showAbout = false;
      else if (showSettings) showSettings = false;
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

  {#if updateAvailable}
    <div in:fly={bannerFlyInParams} out:fly={bannerFlyOutParams}>
      <UpdateBanner
        info={updateAvailable}
        installing={isInstallingUpdate}
        error={updateError}
        onInstall={installAvailableUpdate}
        onDismiss={dismissUpdateBanner}
      />
    </div>
  {/if}

  <main>
    {#if showOnboarding}
      <OnboardingChecklist
        outputDirSet={outputDirConfigured}
        filesReady={filesReady}
        {showInstallHint}
        onDismiss={dismissOnboarding}
        onPickOutput={onboardingPickOutput}
      />
    {/if}

    <div class="section">
      <div class="section-title config-section-header">
        <span>{t("config.title")}</span>
        {#if hasSuccessfulParse}
          <button
            type="button"
            class="config-collapse-btn"
            onclick={toggleConfigCollapsed}
            aria-expanded={!configCollapsed}
          >
            {configCollapsed ? t("config.expand") : t("config.collapse")}
          </button>
        {/if}
      </div>
      {#if !configCollapsed}
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
          {#if ocrEnabled}
            <p class="caption-hint ocr-workers-hint">{t("config.ocrWorkersHint")}</p>
          {/if}
        </div>
      {:else}
        <button
          type="button"
          class="secondary config-collapsed-summary"
          onclick={toggleConfigCollapsed}
        >
          {outputDir ? outputDir : t("config.downloads")} · {format.toUpperCase()}
          {#if ocrEnabled} · OCR{/if}
        </button>
      {/if}
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
            onRerun={rerunBatch}
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
    {#if showAbout}
      <AboutScreen onClose={() => (showAbout = false)} />
    {:else}
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
        onOpenAbout={() => (showAbout = true)}
        {finderActionInstalled}
        {finderActionBusy}
        finderActionNotice={finderActionNotice}
        onInstallFinderAction={installFinderQuickAction}
        {appVersion}
        {updateCheckBusy}
        updateStatusNote={updateStatusNote}
        {updateStatusOk}
        onCheckForUpdates={handleCheckForUpdates}
        onClose={() => (showSettings = false)}
      />
    {/if}
        </div>
      </div>
    {/key}
  {/if}
</div>