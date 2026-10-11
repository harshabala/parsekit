<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import {
    hideProgressHudWindow,
    showProgressHudWindow,
    syncProgressHud,
    type ProgressHudState,
  } from "./lib/progressHud";
  import { downloadDir } from "@tauri-apps/api/path";

  import {
    DEFAULT_TOKEN_STATS_PERIOD,
    getSetting,
    setSetting,
    type TokenStatsPeriod,
  } from "./lib/store";
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
  import { truncatePath } from "./lib/pathDisplay";
  import {
    applyParseProgressEvent,
    applyTokenSavingsEvent,
    createBatchTokenSavings,
    settleBatchOnStop,
    settleRemainingOnDone,
    type BatchTokenSavings,
  } from "./lib/progress";
  import {
    getTokenStats,
    recordTokenSavingsFromSidecarEvent,
    type TokenStats,
  } from "./lib/tokenStats";
  import {
    ACTIVATION_AT_KEY,
    CONVERT_DESTINATION_KEY,
    DESTINATION_PROMPT_KEY,
    qualifiesForActivation,
    type ConvertDestination,
  } from "./lib/activation";
  import {
    loadJobOutcomes,
    persistJobOutcome,
    successRate,
    type JobOutcomesState,
  } from "./lib/jobOutcomes";
  import { applyTheme, DEFAULT_THEME, normalizeThemeMode } from "./lib/theme";
  import DropZone from "./components/DropZone.svelte";
  import OutputFolderPicker from "./components/OutputFolderPicker.svelte";
  import FormatSelector from "./components/FormatSelector.svelte";
  import ProgressList from "./components/ProgressList.svelte";
  import RecentBatches from "./components/RecentBatches.svelte";
  import HistoryScreen from "./components/HistoryScreen.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import AboutScreen from "./components/AboutScreen.svelte";

  import UpdateBanner from "./components/UpdateBanner.svelte";
  import TokenSavingsBanner from "./components/TokenSavingsBanner.svelte";
  import BatchScoreboard from "./components/BatchScoreboard.svelte";
  import { updateState } from "./lib/updateState.svelte";
  import { finderActionState } from "./lib/finderActionState.svelte";
  import { pickOutputFolder } from "./lib/picker";
  import { warmDependencies } from "./lib/depsCache";
  import { DEFAULT_GLOBAL_SHORTCUT } from "./lib/globalShortcut";
  import { isConverterDependencyError, type SettingsTab } from "./lib/converterErrors";
  import LinkIcon from "phosphor-svelte/lib/LinkIcon";
  import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import {
    bannerFlyIn,
    bannerFlyOut,
    buttonFadeInMaybeInstant,
    buttonFadeOutMaybeInstant,
    collapseFadeIn,
    collapseFadeOut,
    hintFadeIn,
    hintFadeOut,
    panelBlurFlyIn,
    panelBlurFlyInParams,
    panelBlurFlyOut,
    panelBlurFlyOutParams,
    panelFadeIn,
    panelFadeOut,
    sectionFlyInMaybeLight,
    sectionFlyOutMaybeLight,
    subviewFadeOut,
  } from "./lib/motion";
  import "./index.css";

  const reducedMotion = $derived(prefersReducedMotion.current);
  const mainFadeIn = $derived(panelFadeIn(reducedMotion));
  const mainFadeOut = $derived(panelFadeOut(reducedMotion));
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));
  const configFadeIn = $derived(collapseFadeIn(reducedMotion));
  const configFadeOut = $derived(collapseFadeOut(reducedMotion));
  const subviewFadeOutParams = $derived(subviewFadeOut(reducedMotion));
  const subviewFadeInParams = $derived(panelFadeIn(reducedMotion));
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
  let lastParsingId = $state<string | null>(null);
  let totalFiles = $state(0);
  let recentBatches = $state<BatchResult[]>([]);
  let showSettings = $state(false);
  let settingsTab = $state<SettingsTab>("general");
  let showAbout = $state(false);
  let showHistory = $state(false);
  let theme = $state<ThemeMode>(DEFAULT_THEME);
  let inputFileCount = $state<number | null>(null);
  let errorMsg = $state<string | null>(null);
  let noticeMsg = $state<string | null>(null);
  let parseRun = $state<ParseRunHandle | null>(null);
  let isIngesting = $state(false);
  let launchAtLogin = $state(false);
  let autoConvertOnCopy = $state(false);
  let globalShortcut = $state(DEFAULT_GLOBAL_SHORTCUT);

  let configCollapsed = $state(false);
  let hasSuccessfulParse = $state(false);
  let appVersion = $state("0.2.0");
  let tokenStats = $state<TokenStats | null>(null);
  let tokenStatsPeriod = $state<TokenStatsPeriod>(DEFAULT_TOKEN_STATS_PERIOD);
  let batchTokenSavings = $state<BatchTokenSavings>(createBatchTokenSavings());
  let showFloatingHud = $state(true);
  let isBackgroundBatch = $state(false);
  let hudActive = $state(false);
  let activatedAt = $state<string | null>(null);
  let jobOutcomes = $state<JobOutcomesState>({ jobs: [] });
  let showDestinationPrompt = $state(false);
  let showBatchScoreboard = $state(false);
  /** Instant chrome for keyboard convert (Emil: keyboard-initiated = no animation). */
  let parseChromeInstant = $state(false);
  /**
   * Instant panel out when Escape closes settings/history/about.
   * Set only via closeSettings/closeHistory (and open*) from the call site:
   * pass `{ instant: true }` for Escape; omit/false for Back so panel out still animates.
   * Do not clear this flag in children after App Escape sets it — use the close helpers.
   */
  let viewChromeInstant = $state(false);
  /** Focus restore target when Settings/History close. */
  let dialogReturnFocus: HTMLElement | null = null;

  function captureDialogReturnFocus() {
    const el = document.activeElement;
    dialogReturnFocus = el instanceof HTMLElement ? el : null;
  }

  function restoreDialogReturnFocus() {
    const el = dialogReturnFocus;
    dialogReturnFocus = null;
    if (el && document.contains(el) && typeof el.focus === "function") {
      requestAnimationFrame(() => el.focus({ preventScroll: true }));
    }
  }
  const lightParseMotion = $derived(parseChromeInstant || isBackgroundBatch);
  const mainPanelIn = $derived(
    panelBlurFlyInParams(reducedMotion, { instant: false }),
  );
  const mainPanelOut = $derived(
    panelBlurFlyOutParams(reducedMotion, { instant: viewChromeInstant }),
  );
  const settingsPanelIn = $derived(
    panelBlurFlyInParams(reducedMotion, {
      instant: viewChromeInstant,
      originScale: true,
    }),
  );
  const settingsPanelOut = $derived(
    panelBlurFlyOutParams(reducedMotion, { instant: viewChromeInstant }),
  );
  const sectionFlyInParams = $derived(
    sectionFlyInMaybeLight(reducedMotion, lightParseMotion),
  );
  const sectionFlyOutParams = $derived(
    sectionFlyOutMaybeLight(reducedMotion, lightParseMotion),
  );
  const buttonFadeInParams = $derived(
    buttonFadeInMaybeInstant(reducedMotion, parseChromeInstant),
  );
  const buttonFadeOutParams = $derived(
    buttonFadeOutMaybeInstant(reducedMotion, parseChromeInstant),
  );

  $effect(() => {
    if (updateState.available) {
      showSettings = false;
      showAbout = false;
    }
  });

  const PARSE_STALL_BASE_MS = 120_000;
  const PARSE_STALL_PER_FILE_MS = 15_000;
  const PARSE_STALL_MAX_MS = 600_000;

  function parseStallTimeoutMs(fileCount: number): number {
    return Math.min(
      PARSE_STALL_MAX_MS,
      PARSE_STALL_BASE_MS + fileCount * PARSE_STALL_PER_FILE_MS
    );
  }

  let parseStallLimitMs = PARSE_STALL_BASE_MS;
  let lastParseEventAt = 0;
  let parseStallTimer: ReturnType<typeof setInterval> | null = null;

  let showProgress = $derived(isParsing || files.length > 0);
  let canRunParse = $derived(
    !isParsing &&
      !!outputDir &&
      inputFileCount !== null &&
      inputFileCount > 0
  );
  /** Why Convert is disabled — shown under the CTA (Emil: disabled must explain itself). */
  let canRunParseBlockedReason = $derived.by(() => {
    if (isParsing) return null;
    if (isIngesting) return t("dropzone.scanning");
    if (!outputDir) return t("errors.needOutputFolder");
    if (inputFileCount === null || inputFileCount <= 0) return t("errors.addFiles");
    return null;
  });


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
        clipboardLabel: t("tray.parseClipboard"),
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

  function openSettings(tab: SettingsTab = "general") {
    viewChromeInstant = false;
    if (!showSettings) captureDialogReturnFocus();
    settingsTab = tab;
    showHistory = false;
    showAbout = false;
    showSettings = true;
    void finderActionState.refreshStatus();
    void refreshTokenStats();
  }

  async function refreshTokenStats() {
    try {
      tokenStats = await getTokenStats();
    } catch (e) {
      console.warn("[tokenStats] refresh failed", e);
    }
  }

  async function handleTokenStatsPeriodChange(period: TokenStatsPeriod) {
    tokenStatsPeriod = period;
    await setSetting("tokenStatsPeriod", period);
  }

  function handleTokenStatsChange(stats: TokenStats) {
    tokenStats = stats;
  }

  function openFileSupportSettings() {
    openSettings("file-support");
  }

  async function ingestExternalPaths(
    paths: string[],
    options?: { openPopover?: boolean }
  ) {
    const openPopover = options?.openPopover ?? true;
    const supported = filterSupportedPaths(paths);
    if (supported.length === 0) {
      if (openPopover) {
        errorMsg = t("errors.noSupported");
      } else {
        void invoke("show_completion_notification", {
          title: t("app.name"),
          body: t("errors.noSupported"),
        }).catch(() => {});
      }
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
    if (openPopover) {
      void openPopoverFromExternal();
    }
  }

  function buildHudState(): ProgressHudState {
    return {
      files,
      total: totalFiles || files.length,
      isParsing,
      batchTokenSavings,
    };
  }

  async function syncHudIfActive() {
    if (!hudActive) return;
    await syncProgressHud(buildHudState());
  }

  async function maybeShowHud() {
    if (!showFloatingHud) return;
    hudActive = true;
    await showProgressHudWindow();
    await syncProgressHud(buildHudState());
    await new Promise((resolve) => setTimeout(resolve, 120));
    await syncHudIfActive();
  }

  async function maybeHideHud() {
    if (!hudActive) return;
    hudActive = false;
    await hideProgressHudWindow();
  }

  async function handleBackgroundParse(paths: string[]) {
    if (!outputDir) {
      void invoke("show_completion_notification", {
        title: t("app.name"),
        body: t("config.outputFolder") + ": " + t("onboarding.stepOutput"),
      }).catch(() => {});
      return;
    }
    const supported = filterSupportedPaths(paths);
    if (supported.length === 0) {
      void invoke("show_completion_notification", {
        title: t("app.name"),
        body: t("errors.noSupported"),
      }).catch(() => {});
      return;
    }
    isBackgroundBatch = true;
    await ingestExternalPaths(supported, { openPopover: false });
    if (!isParsing) {
      void startParse();
    }
  }

  async function handleShowFloatingHudChange(enabled: boolean) {
    showFloatingHud = enabled;
    await setSetting("showFloatingHud", enabled);
    if (!enabled) {
      await maybeHideHud();
    }
  }

  async function handleGlobalShortcutChange(shortcut: string) {
    globalShortcut = shortcut;
  }

  async function openPopoverFromExternal() {
    try {
      await invoke("show_main_window");
    } catch {
      /* dev / web */
    }
  }

  function openHistory() {
    viewChromeInstant = false;
    if (!showHistory) captureDialogReturnFocus();
    showSettings = false;
    showAbout = false;
    showHistory = true;
  }

  function closeHistory(options?: { instant?: boolean }) {
    viewChromeInstant = options?.instant === true;
    showHistory = false;
    restoreDialogReturnFocus();
  }

  function closeSettings(options?: { instant?: boolean }) {
    viewChromeInstant = options?.instant === true;
    showAbout = false;
    showSettings = false;
    restoreDialogReturnFocus();
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

  onMount(() => {
    let unlistenOpen: (() => void) | undefined;
    let unlistenBackgroundParse: (() => void) | undefined;
    let unlistenHudFileSupport: (() => void) | undefined;

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
    // OCR is automatic for scans (main panel no longer exposes a toggle).
    ocrEnabled = true;
    await setSetting("ocrEnabled", true);
    const rawOcr = String(await getSetting("ocrLanguage", "eng"));
    if (!isKnownOcrLanguage(rawOcr)) {
      noticeMsg = t("settings.ocrMigrated");
    }
    ocrLanguage = normalizeOcrLanguage(rawOcr);
    await setSetting("ocrLanguage", ocrLanguage);
    recentBatches = await getSetting<BatchResult[]>("recentBatches", []);
    hasSuccessfulParse = await getSetting("hasSuccessfulParse", false);
    configCollapsed = hasSuccessfulParse;
    activatedAt = await getSetting<string | null>(ACTIVATION_AT_KEY, null);
    jobOutcomes = await loadJobOutcomes();
    // Popover opens from tray click after onboarding — not auto-shown on launch.
    await resolveDefaultWorkers(await getSetting<number>("workers", 0));
    launchAtLogin = await getSetting<boolean>("launchAtLogin", false);
    autoConvertOnCopy = await getSetting<boolean>("autoConvertOnCopy", false);
    globalShortcut = await getSetting("globalShortcut", DEFAULT_GLOBAL_SHORTCUT);
    showFloatingHud = await getSetting("showFloatingHud", true);
    if (launchAtLogin) {
      try {
        await invoke("set_launch_at_login", { enabled: true });
      } catch {
        launchAtLogin = false;
        await setSetting("launchAtLogin", false);
      }
    }
    try {
      await invoke("set_auto_convert_on_copy", { enabled: autoConvertOnCopy });
    } catch {
      /* tray/native only */
    }
    await syncTrayMenu();

    tokenStatsPeriod = await getSetting<TokenStatsPeriod>(
      "tokenStatsPeriod",
      DEFAULT_TOKEN_STATS_PERIOD,
    );
    if (
      tokenStatsPeriod !== "today" &&
      tokenStatsPeriod !== "month" &&
      tokenStatsPeriod !== "lifetime"
    ) {
      tokenStatsPeriod = DEFAULT_TOKEN_STATS_PERIOD;
      await setSetting("tokenStatsPeriod", tokenStatsPeriod);
    }
    await refreshTokenStats();

    try {
      const info = await invoke<{ version?: string }>("get_system_info");
      if (info.version) appVersion = info.version;
    } catch {
      /* keep default */
    }

    updateState.scheduleBackgroundCheck(appVersion);

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

    unlistenBackgroundParse = await listen<string[]>("background-parse", (event) => {
      const paths = event.payload;
      if (paths?.length) {
        void handleBackgroundParse(paths);
      }
    });

    unlistenHudFileSupport = await listen("hud-open-file-support", () => {
      openFileSupportSettings();
    });
    })();

    return () => {
      unlistenOpen?.();
      unlistenBackgroundParse?.();
      unlistenHudFileSupport?.();
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

  function toggleConfigCollapsed() {
    configCollapsed = !configCollapsed;
  }

  async function handleFormatChange(f: OutputFormat) {
    format = f;
    await setSetting("format", format);
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

  async function handleAutoConvertOnCopyChange(enabled: boolean) {
    autoConvertOnCopy = enabled;
    await setSetting("autoConvertOnCopy", enabled);
    try {
      await invoke("set_auto_convert_on_copy", { enabled });
    } catch {
      /* native only */
    }
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
      if (Date.now() - lastParseEventAt < parseStallLimitMs) return;
      parseRun?.cancel();
      parseRun = null;
      stopParseUi(t("errors.engineStalled"), t("errors.engineStalled"));
    }, 5000);
  }

  function stopParseUi(notice: string, error: string | null = null) {
    clearParseStallWatchdog();
    isParsing = false;
    files = settleBatchOnStop(files, {
      parsing: t("errors.batchInterrupted"),
      pending: t("errors.batchNotReached"),
    });
    lastParsingId = null;
    errorMsg = error;
    noticeMsg = notice;
    void syncHudIfActive();
    isBackgroundBatch = false;
    parseChromeInstant = false;
  }

  function cancelParse() {
    parseRun?.cancel();
    parseRun = null;
    stopParseUi(t("errors.parseCancelled"));
  }

  async function startParse(options?: { instantChrome?: boolean }) {
    if (!outputDir || inputFileCount === 0) return;

    parseChromeInstant = options?.instantChrome === true;
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

    filesToParse = [...new Map(filesToParse.map((p) => [p, p])).values()];
    try {
      filesToParse = await invoke<string[]>("canonicalize_paths", {
        paths: filesToParse,
      });
    } catch (e) {
      console.warn("[canonicalize_paths]", e);
    }

    try {
      await invoke("trigger_haptic");
    } catch {}

    isParsing = true;
    parseStallLimitMs = parseStallTimeoutMs(filesToParse.length);
    startParseStallWatchdog();
    lastParsingId = null;
    batchTokenSavings = createBatchTokenSavings();
    showBatchScoreboard = false;
    showDestinationPrompt = false;
    totalFiles = filesToParse.length;
    files = filesToParse.map((path) => ({
      id: path,
      name: fileBaseName(path),
      status: "pending" as const,
    }));

    await maybeShowHud();

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
          } else if (event.type === "token_savings") {
            const savingsEvent = {
              type: "token_savings" as const,
              file: event.file,
              file_type: event.file_type,
              tokens_saved: event.tokens_saved,
              pages_unlocked: event.pages_unlocked,
              documents_unlocked: event.documents_unlocked,
            };
            batchTokenSavings = applyTokenSavingsEvent(batchTokenSavings, savingsEvent);
            void syncHudIfActive();
            void recordTokenSavingsFromSidecarEvent(savingsEvent)
              .then((next) => {
                if (next) tokenStats = next;
              })
              .catch((e) => console.warn("[tokenStats] record failed", e));
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
            void syncHudIfActive();
          } else if (event.type === "done") {
            clearParseStallWatchdog();
            isParsing = false;
            lastParsingId = null;
            totalFiles = totalFiles || files.length;
            const sawProgress = files.some((f) => f.status !== "pending");
            if (!sawProgress) {
              errorMsg = t("errors.engineNoOutput");
              noticeMsg = t("errors.parseFailed");
              files = settleBatchOnStop(files, {
                parsing: t("errors.engineNoOutput"),
                pending: t("errors.engineNoOutput"),
              });
              return;
            }
            files = settleRemainingOnDone(files, t("errors.batchNotReached"));
            const errCount = files.filter((f) => f.status === "error").length;
            if (errCount > 0) {
              noticeMsg = t("run.batchDoneWithErrors", { errors: errCount });
              errorMsg = null;
            } else {
              noticeMsg = null;
              errorMsg = null;
            }
            void addToHistory();
            const parsed = files.filter((f) => f.status === "done").length;
            const notifyErrors = files.filter((f) => f.status === "error").length;
            if (!hasSuccessfulParse && parsed > 0) {
              hasSuccessfulParse = true;
              configCollapsed = true;
              void setSetting("hasSuccessfulParse", true);
            }
            // PK-2/3: local job outcomes + activation (no paths stored)
            const tokensInBatch = batchTokenSavings.tokensSaved;
            void persistJobOutcome({
              success: parsed,
              failed: notifyErrors,
              tokensSaved: tokensInBatch,
            }).then((next) => {
              jobOutcomes = next;
            });
            if (
              qualifiesForActivation({
                alreadyActivated: !!activatedAt,
                successCount: parsed,
                tokensSavedInBatch: tokensInBatch,
              })
            ) {
              const at = new Date().toISOString();
              activatedAt = at;
              void setSetting(ACTIVATION_AT_KEY, at);
            }
            // PK-4: soft destination prompt once after first success
            void getSetting(DESTINATION_PROMPT_KEY, false).then((seen) => {
              showDestinationPrompt = !seen && parsed > 0;
            });
            showBatchScoreboard = parsed > 0 || notifyErrors > 0;
            void refreshTokenStats();
            void syncHudIfActive();
            void invoke("show_completion_notification", {
              title: t("app.name"),
              body: t("run.notifyDone", { parsed, errors: notifyErrors }),
            }).catch(() => {});
            isBackgroundBatch = false;
          } else if (event.type === "error") {
            // Fatal sidecar error (see sidecar.ts protocol) — not per-file progress errors.
            console.error("[parse batch]", event.message);
            if (isParsing && event.message) {
              parseRun?.cancel();
              parseRun = null;
              stopParseUi(t("errors.parseFailed"), event.message);
            } else if (event.message) {
              errorMsg = event.message;
            }
          }
        }
    );

    try {
      await parseRun.promise;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      const allSettled = files.every(
        (f) =>
          f.status === "done" ||
          f.status === "error" ||
          f.status === "skipped"
      );
      if (isParsing) {
        if (allSettled) {
          clearParseStallWatchdog();
          isParsing = false;
          lastParsingId = null;
          const errCount = files.filter((f) => f.status === "error").length;
          noticeMsg =
            errCount > 0
              ? t("run.batchDoneWithErrors", { errors: errCount })
              : null;
          errorMsg = errCount > 0 ? msg : null;
          void addToHistory();
          void syncHudIfActive();
          isBackgroundBatch = false;
        } else {
          stopParseUi(t("errors.parseFailed"), msg);
        }
        console.error(e);
      }
    } finally {
      clearParseStallWatchdog();
      parseRun = null;
    }
  }

  async function addToHistory() {
    const parsed = files.filter((f) => f.status === "done").length;
    const errored = files.filter((f) => f.status === "error");
    const errors = errored.length;
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
      fileErrors:
        errors > 0
          ? errored.map((f) => ({
              file: f.name,
              error: f.error ?? t("errors.parseFailed"),
            }))
          : undefined,
    };
    recentBatches = [newBatch, ...recentBatches.slice(0, MAX_RECENT_BATCHES - 1)];
    await setSetting("recentBatches", recentBatches);
  }

  async function openFolder(path: string) {
    await invoke("open_in_finder", { path });
  }

  function handleConvertMore() {
    showBatchScoreboard = false;
    files = [];
    lastParsingId = null;
    batchTokenSavings = createBatchTokenSavings();
    selectedFiles = [];
    inputDir = "";
    inputFileCount = null;
    errorMsg = null;
    noticeMsg = null;
  }

  async function handleDestination(value: ConvertDestination) {
    showDestinationPrompt = false;
    await setSetting(DESTINATION_PROMPT_KEY, true);
    await setSetting(CONVERT_DESTINATION_KEY, value);
  }

  const lastBatchSuccessCount = $derived(
    files.filter((f) => f.status === "done").length,
  );
  const lastBatchFailedCount = $derived(
    files.filter((f) => f.status === "error").length,
  );
  const jobSuccessRateLabel = $derived(successRate(jobOutcomes).label);

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "r") {
      e.preventDefault();
      if (canRunParse) {
        void startParse({ instantChrome: true });
      }
    }
    if (e.key === "Escape") {
      // Keyboard-initiated dismiss: no panel fly (Emil frequency rule).
      // Use close*({ instant: true }) so child Escape handlers cannot clear the flag.
      if (showAbout) {
        showAbout = false;
      } else if (showSettings) {
        closeSettings({ instant: true });
      } else if (showHistory) {
        closeHistory({ instant: true });
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="shell">
  {#if !showSettings && !showHistory}
    {#key "main"}
      <div class="motion-panel" in:panelBlurFlyIn={mainPanelIn} out:panelBlurFlyOut={mainPanelOut}>
        <div class="motion-panel-content">
  <header class="main-header">
    <div class="main-header-brand">
      <span class="main-header-title">{t("app.name")}</span>
      {#if appVersion}<span class="header-ver">{appVersion}</span>{/if}
    </div>
    <div class="header-actions">
      <button
        type="button"
        class="icon-btn icon-btn-settings"
        onclick={() => openSettings()}
        onmouseenter={warmDependencies}
        onfocusin={warmDependencies}
        title={t("header.settings")}
        aria-label={t("header.settings")}
      >
        <GearSixIcon size={16} weight="regular" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="icon-btn"
        onclick={quitApp}
        title={t("header.quit")}
        aria-label={t("header.quit")}
      >
        <XIcon size={16} weight="bold" aria-hidden="true" />
      </button>
    </div>
  </header>

  {#if updateState.available}
    <div in:fly={bannerFlyInParams} out:fly={bannerFlyOutParams}>
      <UpdateBanner
        info={updateState.available}
        installing={updateState.isInstalling}
        error={updateState.error}
        onInstall={() => updateState.installAvailable()}
        onDismiss={() => updateState.dismiss()}
      />
    </div>
  {/if}

  <main class="main-panel">
    <section class="section output-settings-section">
      {#if hasSuccessfulParse}
        <div class="config-section-header output-settings-toolbar">
          <div class="output-settings-heading">
            <LinkIcon size={16} weight="regular" aria-hidden="true" />
            <span class="output-settings-title">{t("config.title")}</span>
          </div>
          <button
            type="button"
            class="config-collapse-btn"
            in:fade={hintFadeInParams}
            out:fade={hintFadeOutParams}
            onclick={toggleConfigCollapsed}
            aria-expanded={!configCollapsed}
          >
            {configCollapsed ? t("config.expand") : t("config.collapse")}
          </button>
        </div>
      {/if}

      {#if !configCollapsed}
        <div class="card output-settings-card" in:fade={configFadeIn} out:fade={configFadeOut}>
          {#if !hasSuccessfulParse}
            <div class="output-settings-heading">
              <LinkIcon size={16} weight="regular" aria-hidden="true" />
              <span class="output-settings-title">{t("config.title")}</span>
            </div>
          {/if}

          <OutputFolderPicker value={outputDir} onSelect={handleOutputSelect} />

          <div class="config-control-row">
            <div class="config-control-label">
              <FileTextIcon size={16} weight="regular" aria-hidden="true" />
              <span>{t("config.format")}</span>
            </div>
            <div class="config-control-fields config-control-fields--format">
              <FormatSelector value={format} onChange={handleFormatChange} />
              <span class="ocr-auto-chip" title={t("config.ocrAutoHint")}>
                <SparkleIcon size={12} weight="fill" aria-hidden="true" />
                {t("config.ocrAuto")}
              </span>
            </div>
          </div>

          <p class="caption-hint output-settings-hint">{t("config.formatHint")}</p>
        </div>
      {:else}
        <button
          type="button"
          class="config-collapsed-summary"
          title={outputDir || t("config.downloads")}
          in:fade={configFadeIn}
          out:fade={configFadeOut}
          onclick={toggleConfigCollapsed}
        >
          <span class="config-summary-path"
            >{outputDir ? truncatePath(outputDir, 26) : t("config.downloads")}</span
          >
          <span class="config-summary-chips">
            <span class="config-summary-chip">{format.toUpperCase()}</span>
            <span class="config-summary-chip">OCR</span>
          </span>
        </button>
      {/if}
    </section>

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
          lightMotion={lightParseMotion}
          onOpenFileSupport={openFileSupportSettings}
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
              onclick={() => void startParse()}
            >
              {t("run.runParse")}
            </button>
          {/if}
        </div>
      {/key}
      {#if canRunParseBlockedReason}
        <p class="run-blocked-hint" role="status">{canRunParseBlockedReason}</p>
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
          <p class="error-banner-text">{errorMsg}</p>
          {#if isConverterDependencyError(errorMsg)}
            <button
              type="button"
              class="error-banner-link"
              onclick={openFileSupportSettings}
            >
              {t("errors.openFileSupport")}
            </button>
          {/if}
        </div>
      {/if}
      {#if !isParsing && showBatchScoreboard && files.length > 0}
        <BatchScoreboard
          batch={batchTokenSavings}
          stats={tokenStats}
          successCount={lastBatchSuccessCount}
          failedCount={lastBatchFailedCount}
          successRateLabel={jobSuccessRateLabel}
          showDestinationPrompt={showDestinationPrompt}
          onOpenOutput={() => openFolder(outputDir)}
          onConvertMore={handleConvertMore}
          onDestination={handleDestination}
        />
      {:else if !isParsing && files.length > 0 && files.some((f) => f.status === "done")}
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

      {#if !isParsing && !hasSuccessfulParse && !showBatchScoreboard}
        <div class="first-run-hero" in:fly={sectionFlyInParams}>
          <p class="first-run-hero-title">{t("firstRun.title")}</p>
          <p class="first-run-hero-body">{t("firstRun.body")}</p>
          <p class="first-run-hero-privacy">{t("scoreboard.privacy")}</p>
        </div>
      {/if}
    </div>

    {#if !isParsing}
      <div in:fade={mainFadeIn} out:fade={mainFadeOut}>
        <TokenSavingsBanner
          stats={tokenStats}
          period={tokenStatsPeriod}
          onOpenDetails={() => openSettings("general")}
        />
        <RecentBatches
          {latestBatch}
          {appVersion}
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
      <div class="motion-panel" in:panelBlurFlyIn={mainPanelIn} out:panelBlurFlyOut={mainPanelOut}>
        <div class="motion-panel-content">
          <HistoryScreen
            batches={recentBatches}
            onOpenFolder={openFolder}
            onRerun={rerunBatch}
            {appVersion}
            onClose={closeHistory}
          />
        </div>
      </div>
    {/key}
  {/if}

  {#if showSettings}
    {#key "settings"}
      <div
        class="motion-panel motion-panel--settings"
        in:panelBlurFlyIn={settingsPanelIn}
        out:panelBlurFlyOut={settingsPanelOut}
      >
        <div class="motion-panel-content">
          {#key showAbout}
            <div class="subview-fill">
              {#if showAbout}
                <div
                  class="subview-pane"
                  in:fade={subviewFadeInParams}
                  out:fade={subviewFadeOutParams}
                >
                  <AboutScreen onClose={() => (showAbout = false)} />
                </div>
              {:else}
                <div
                  class="subview-pane"
                  in:fade={subviewFadeInParams}
                  out:fade={subviewFadeOutParams}
                >
                  <SettingsScreen
                    locale={getLocale()}
                    {ocrLanguage}
                    {ocrEnabled}
                    {theme}
                    {workers}
                    {launchAtLogin}
                    {autoConvertOnCopy}
                    {globalShortcut}
                    {showFloatingHud}
                    initialTab={settingsTab}
                    tokenStats={tokenStats}
                    tokenStatsPeriod={tokenStatsPeriod}
                    onLocaleChange={handleLocaleChange}
                    onOcrLanguageChange={handleOcrLanguageChange}
                    onThemeChange={handleThemeChange}
                    onWorkersChange={handleWorkersChange}
                    onLaunchAtLoginChange={handleLaunchAtLoginChange}
                    onAutoConvertOnCopyChange={handleAutoConvertOnCopyChange}
                    onGlobalShortcutChange={handleGlobalShortcutChange}
                    onShowFloatingHudChange={handleShowFloatingHudChange}
                    onTokenStatsPeriodChange={handleTokenStatsPeriodChange}
                    onTokenStatsChange={handleTokenStatsChange}
                    onOpenAbout={() => (showAbout = true)}
                    finderActionInstalled={finderActionState.installed}
                    finderActionBusy={finderActionState.busy}
                    finderActionNotice={finderActionState.notice}
                    onInstallFinderAction={() => finderActionState.install()}
                    {appVersion}
                    updateCheckBusy={updateState.checkBusy}
                    updateStatusNote={updateState.statusNote}
                    updateStatusOk={updateState.statusOk}
                    onCheckForUpdates={() => updateState.checkForUpdates(appVersion)}
                    onClose={closeSettings}
                  />
                </div>
              {/if}
            </div>
          {/key}
        </div>
      </div>
    {/key}
  {/if}
</div>