<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly, slide } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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
  import TokenSavingsBanner from "./components/TokenSavingsBanner.svelte";
  import { updateState } from "./lib/updateState.svelte";
  import { finderActionState } from "./lib/finderActionState.svelte";
  import { pickOutputFolder } from "./lib/picker";
  import { warmDependencies } from "./lib/depsCache";
  import { DEFAULT_GLOBAL_SHORTCUT } from "./lib/globalShortcut";
  import { isConverterDependencyError, type SettingsTab } from "./lib/converterErrors";
  import {
    bannerFlyIn,
    bannerFlyOut,
    buttonFadeIn,
    buttonFadeOut,
    collapseSlideIn,
    collapseSlideOut,
    hintFadeIn,
    hintFadeOut,
    panelBlurFlyIn,
    panelBlurFlyInParams,
    panelBlurFlyOut,
    panelBlurFlyOutParams,
    panelFadeIn,
    panelFadeOut,
    sectionFlyIn,
    sectionFlyOut,
    subviewFadeOut,
  } from "./lib/motion";
  import "./index.css";

  const reducedMotion = $derived(prefersReducedMotion.current);
  const mainPanelIn = $derived(panelBlurFlyInParams(reducedMotion));
  const mainPanelOut = $derived(panelBlurFlyOutParams(reducedMotion));
  const mainFadeIn = $derived(panelFadeIn(reducedMotion));
  const mainFadeOut = $derived(panelFadeOut(reducedMotion));
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));
  const configSlideIn = $derived(collapseSlideIn(reducedMotion));
  const configSlideOut = $derived(collapseSlideOut(reducedMotion));
  const subviewFadeOutParams = $derived(subviewFadeOut(reducedMotion));
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
  let showOnboarding = $state(false);
  let showInstallHint = $state(false);
  let configCollapsed = $state(false);
  let hasSuccessfulParse = $state(false);
  let appVersion = $state("0.2.0");
  let tokenStats = $state<TokenStats | null>(null);
  let tokenStatsPeriod = $state<TokenStatsPeriod>(DEFAULT_TOKEN_STATS_PERIOD);
  let batchTokenSavings = $state<BatchTokenSavings>(createBatchTokenSavings());
  let showFloatingHud = $state(true);
  let isBackgroundBatch = $state(false);
  let hudActive = $state(false);

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
    if (tokenStatsPeriod !== "month" && tokenStatsPeriod !== "lifetime") {
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
            if (!hasSuccessfulParse) {
              hasSuccessfulParse = true;
              configCollapsed = true;
              void setSetting("hasSuccessfulParse", true);
            }
            const parsed = files.filter((f) => f.status === "done").length;
            const notifyErrors = files.filter((f) => f.status === "error").length;
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
    try {
      await invoke("open_in_finder", { path });
    } catch {
      const { Command } = await import("@tauri-apps/plugin-shell");
      await Command.create("open", [path]).spawn();
    }
  }

  function buildErrorReport(batch: BatchResult): string {
    const lines = [
      "# ParseKit error report",
      "",
      `- Date: ${batch.timestamp}`,
      `- ParseKit version: ${appVersion}`,
      `- Output format: ${batch.format.toUpperCase()}`,
      `- Files: ${batch.fileCount} · Parsed: ${batch.parsed} · Errors: ${batch.errors}`,
      `- Output folder: ${batch.outputDir}`,
      "",
      "## Failed files",
      "",
    ];
    (batch.fileErrors ?? []).forEach((fe, i) => {
      lines.push(`${i + 1}. ${fe.file}`);
      lines.push(`   ${fe.error}`);
      lines.push("");
    });
    return lines.join("\n");
  }

  async function saveErrorReport(batch: BatchResult) {
    if (!batch.fileErrors || batch.fileErrors.length === 0) return;
    const stamp = batch.timestamp.replace(/[:.]/g, "-").slice(0, 19);
    const fileName = `parsekit-errors-${stamp}.md`;
    try {
      await invoke<string>("save_error_report", {
        dir: batch.outputDir,
        fileName,
        contents: buildErrorReport(batch),
      });
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
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
      <div class="motion-panel" in:panelBlurFlyIn={mainPanelIn} out:panelBlurFlyOut={mainPanelOut}>
        <div class="motion-panel-content">
  <header>
    <span>{t("app.name")}</span>
    {#if appVersion}<span class="header-ver">{appVersion}</span>{/if}
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
        type="button"
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

  <main>
    {#if showOnboarding}
      <div in:fly={sectionFlyInParams} out:fly={sectionFlyOutParams}>
        <OnboardingChecklist
          outputDirSet={outputDirConfigured}
          filesReady={filesReady}
          {showInstallHint}
          onDismiss={dismissOnboarding}
          onPickOutput={onboardingPickOutput}
        />
      </div>
    {/if}

    <div class="section">
      <div class="section-title config-section-header">
        <span>{t("config.title")}</span>
        {#if hasSuccessfulParse}
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
        {/if}
      </div>
      {#if !configCollapsed}
        <div class="card" in:slide={configSlideIn} out:slide={configSlideOut}>
          <OutputFolderPicker value={outputDir} onSelect={handleOutputSelect} />

          <div class="row">
            <span>{t("config.format")}</span>
            <FormatSelector value={format} onChange={handleFormatChange} />
          </div>
          {#if format !== "json"}
            <div
              class="file-count-preview caption-hint"
              in:fade={hintFadeInParams}
              out:fade={hintFadeOutParams}
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
            <p
              class="caption-hint ocr-workers-hint"
              in:fade={hintFadeInParams}
              out:fade={hintFadeOutParams}
            >
              {t("config.ocrWorkersHint")}
            </p>
          {/if}
        </div>
      {:else}
        <button
          type="button"
          class="config-collapsed-summary"
          title={outputDir || t("config.downloads")}
          in:slide={configSlideIn}
          out:slide={configSlideOut}
          onclick={toggleConfigCollapsed}
        >
          <span class="config-summary-path"
            >{outputDir ? truncatePath(outputDir, 26) : t("config.downloads")}</span
          >
          <span class="config-summary-chips">
            <span class="config-summary-chip">{format.toUpperCase()}</span>
            {#if ocrEnabled}<span class="config-summary-chip">OCR</span>{/if}
          </span>
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
        <TokenSavingsBanner
          stats={tokenStats}
          period={tokenStatsPeriod}
          onOpenDetails={() => openSettings("general")}
        />
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
      <div class="motion-panel" in:panelBlurFlyIn={mainPanelIn} out:panelBlurFlyOut={mainPanelOut}>
        <div class="motion-panel-content">
          <HistoryScreen
            batches={recentBatches}
            onOpenFolder={openFolder}
            onRerun={rerunBatch}
            onSaveErrors={saveErrorReport}
            onClose={() => (showHistory = false)}
          />
        </div>
      </div>
    {/key}
  {/if}

  {#if showSettings}
    {#key "settings"}
      <div class="motion-panel" in:panelBlurFlyIn={mainPanelIn} out:panelBlurFlyOut={mainPanelOut}>
        <div class="motion-panel-content">
    {#key showAbout}
      <div class="subview-fill" out:fade={subviewFadeOutParams}>
    {#if showAbout}
      <div in:fade={mainFadeIn} out:fade={mainFadeOut}>
      <AboutScreen onClose={() => (showAbout = false)} />
      </div>
    {:else}
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
        onClose={() => (showSettings = false)}
      />
    {/if}
      </div>
    {/key}
        </div>
      </div>
    {/key}
  {/if}
</div>