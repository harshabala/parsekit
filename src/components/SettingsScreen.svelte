<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import type { AppLocale } from "../lib/i18n.svelte";
  import { t } from "../lib/i18n.svelte";
  import { hintFadeIn, hintFadeOut, panelFadeIn, panelFadeOut } from "../lib/motion";
  import type { OcrLanguageCode } from "../lib/ocrLanguages";
  import type { ThemeMode } from "../lib/types";
  import type { SettingsTab } from "../lib/converterErrors";
  import LanguageSelector from "./LanguageSelector.svelte";
  import OcrLanguageSelector from "./OcrLanguageSelector.svelte";
  import ThemeSelector from "./ThemeSelector.svelte";
  import WorkersSlider from "./WorkersSlider.svelte";
  import DependencyPreflight from "./DependencyPreflight.svelte";
  import TokenSavingsPanel from "./TokenSavingsPanel.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { TokenStats } from "../lib/tokenStats";
  import type { TokenStatsPeriod } from "../lib/store";
  import {
    DEFAULT_GLOBAL_SHORTCUT,
    formatShortcutDisplay,
    keyboardEventToShortcut,
  } from "../lib/globalShortcut";
  import { setSetting } from "../lib/store";

  let {
    locale: localeValue,
    ocrLanguage,
    ocrEnabled,
    theme,
    workers,
    launchAtLogin,
    autoConvertOnCopy = false,
    globalShortcut = DEFAULT_GLOBAL_SHORTCUT,
    showFloatingHud = true,
    onLocaleChange,
    onOcrLanguageChange,
    onThemeChange,
    onWorkersChange,
    onLaunchAtLoginChange,
    onAutoConvertOnCopyChange,
    onGlobalShortcutChange,
    onShowFloatingHudChange,
    tokenStats = null,
    tokenStatsPeriod = "month",
    onTokenStatsPeriodChange,
    onTokenStatsChange,
    onOpenAbout,
    finderActionInstalled = false,
    finderActionBusy = false,
    finderActionNotice = null,
    onInstallFinderAction,
    appVersion = "0.2.0",
    updateCheckBusy = false,
    updateStatusNote = null,
    updateStatusOk = false,
    onCheckForUpdates,
    initialTab = "general",
    onClose,
  }: {
    locale: AppLocale;
    ocrLanguage: OcrLanguageCode;
    ocrEnabled: boolean;
    theme: ThemeMode;
    workers: number;
    launchAtLogin: boolean;
    autoConvertOnCopy?: boolean;
    globalShortcut?: string;
    showFloatingHud?: boolean;
    onLocaleChange: (code: AppLocale) => void;
    onOcrLanguageChange: (code: OcrLanguageCode) => void;
    onThemeChange: (mode: ThemeMode) => void;
    onWorkersChange: (value: number) => void;
    onLaunchAtLoginChange: (enabled: boolean) => void;
    onAutoConvertOnCopyChange?: (enabled: boolean) => void | Promise<void>;
    onGlobalShortcutChange?: (shortcut: string) => void | Promise<void>;
    onShowFloatingHudChange?: (enabled: boolean) => void | Promise<void>;
    tokenStats?: TokenStats | null;
    tokenStatsPeriod?: TokenStatsPeriod;
    onTokenStatsPeriodChange?: (period: TokenStatsPeriod) => void;
    onTokenStatsChange?: (stats: TokenStats) => void;
    onOpenAbout: () => void;
    finderActionInstalled?: boolean;
    finderActionBusy?: boolean;
    finderActionNotice?: string | null;
    onInstallFinderAction?: () => void;
    appVersion?: string;
    updateCheckBusy?: boolean;
    updateStatusNote?: string | null;
    updateStatusOk?: boolean;
    onCheckForUpdates?: () => void;
    initialTab?: SettingsTab;
    onClose: () => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));
  const tabFadeInParams = $derived(panelFadeIn(reducedMotion));
  const tabFadeOutParams = $derived(panelFadeOut(reducedMotion));

  let activeTab = $state<SettingsTab>("general");
  let gatekeeperCopied = $state(false);
  let gatekeeperCopyError = $state<string | null>(null);
  let recordingHotkey = $state(false);
  let hotkeyError = $state<string | null>(null);
  const hotkeyDisplay = $derived(formatShortcutDisplay(globalShortcut));

  const tabs: { id: SettingsTab; labelKey: string }[] = [
    { id: "general", labelKey: "settings.tabGeneral" },
    { id: "file-support", labelKey: "settings.tabFileSupport" },
  ];

  $effect(() => {
    activeTab = initialTab;
  });

  function handleKeydown(e: KeyboardEvent) {
    if (recordingHotkey) {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        recordingHotkey = false;
        hotkeyError = null;
        return;
      }
      const shortcut = keyboardEventToShortcut(e);
      if (!shortcut || !onGlobalShortcutChange) return;
      recordingHotkey = false;
      void applyHotkeyChange(shortcut);
      return;
    }
    if (e.key === "Escape") {
      onClose();
    }
  }

  async function applyHotkeyChange(shortcut: string) {
    hotkeyError = null;
    try {
      await invoke("update_global_shortcut", { shortcut });
      await setSetting("globalShortcut", shortcut);
      await onGlobalShortcutChange?.(shortcut);
      try {
        await invoke("trigger_haptic");
      } catch {
        /* optional */
      }
    } catch (e) {
      hotkeyError =
        e instanceof Error ? e.message : String(e) || t("settings.hotkeyUpdateFailed");
    }
  }

  function startHotkeyRecording() {
    if (!onGlobalShortcutChange) return;
    hotkeyError = null;
    recordingHotkey = true;
  }

  function resetHotkey() {
    void applyHotkeyChange(DEFAULT_GLOBAL_SHORTCUT);
  }

  async function copyGatekeeperCommand() {
    gatekeeperCopyError = null;
    try {
      const cmd = await invoke<string>("gatekeeper_fix_command");
      await invoke("copy_text_to_clipboard", { text: cmd });
      try {
        await invoke("trigger_haptic");
      } catch {
        /* optional */
      }
      gatekeeperCopied = true;
      setTimeout(() => {
        gatekeeperCopied = false;
      }, 2500);
    } catch (e) {
      gatekeeperCopied = false;
      gatekeeperCopyError =
        e instanceof Error ? e.message : String(e) || t("gatekeeper.copyFailed");
    }
  }

  async function openPrivacySettings() {
    try {
      await invoke("open_privacy_security_settings");
    } catch {
      /* dev */
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-screen" role="dialog" aria-modal="true" aria-labelledby="settings-title">
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={onClose}>{t("settings.back")}</button>
    <span class="settings-header-title" id="settings-title">{t("settings.title")}</span>
  </div>

  <div
    class="settings-tab-bar"
    role="tablist"
    aria-label={t("settings.title")}
  >
    <div class="segmented-control settings-tabs">
      {#each tabs as tab}
        <button
          type="button"
          role="tab"
          id="settings-tab-{tab.id}"
          class="segment"
          class:active={activeTab === tab.id}
          aria-selected={activeTab === tab.id}
          aria-controls="settings-panel-{tab.id}"
          onclick={() => (activeTab = tab.id)}
        >
          {t(tab.labelKey)}
        </button>
      {/each}
    </div>
  </div>

  <div class="settings-scroll selectable-content">
    {#key activeTab}
      {#if activeTab === "general"}
      <div
        id="settings-panel-general"
        role="tabpanel"
        aria-labelledby="settings-tab-general"
        in:fade={tabFadeInParams}
        out:fade={tabFadeOutParams}
      >
        <div class="settings-section">
          <div class="settings-section-title">{t("settings.appLanguageTitle")}</div>
          <p class="settings-hint">{t("settings.appLanguageHint")}</p>
          <LanguageSelector value={localeValue} onChange={onLocaleChange} />
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section">
          <div class="settings-section-title">{t("settings.appearanceTitle")}</div>
          <p class="settings-hint">{t("settings.appearanceHint")}</p>
          <ThemeSelector value={theme} onChange={onThemeChange} />
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section settings-toggle-row">
          <label class="settings-launch-label">
            <input
              type="checkbox"
              checked={launchAtLogin}
              onchange={(e) => onLaunchAtLoginChange((e.currentTarget as HTMLInputElement).checked)}
            />
            <span>{t("settings.launchAtLogin")}</span>
          </label>
          <p class="settings-hint">{t("settings.launchAtLoginHint")}</p>
        </div>

        <div class="settings-divider"></div>

        {#if onAutoConvertOnCopyChange}
          <div class="settings-section settings-toggle-row">
            <label class="settings-launch-label">
              <input
                type="checkbox"
                checked={autoConvertOnCopy}
                onchange={(e) =>
                  onAutoConvertOnCopyChange(
                    (e.currentTarget as HTMLInputElement).checked,
                  )}
              />
              <span>{t("settings.autoConvertOnCopy")}</span>
            </label>
            <p class="settings-hint">{t("settings.autoConvertOnCopyHint")}</p>
          </div>
          <div class="settings-divider"></div>
        {/if}

        {#if onShowFloatingHudChange}
          <div class="settings-section settings-toggle-row">
            <label class="settings-launch-label">
              <input
                type="checkbox"
                checked={showFloatingHud}
                onchange={(e) =>
                  onShowFloatingHudChange((e.currentTarget as HTMLInputElement).checked)}
              />
              <span>{t("settings.floatingHudTitle")}</span>
            </label>
            <p class="settings-hint">{t("settings.floatingHudHint")}</p>
          </div>
          <div class="settings-divider"></div>
        {/if}

        {#if onGlobalShortcutChange}
          <div class="settings-section">
            <div class="settings-section-title">{t("settings.hotkeyTitle")}</div>
            <p class="settings-hint">{t("settings.hotkeyHint")}</p>
            <p class="settings-hint">{t("settings.clipboardConvertHint")}</p>
            <div class="hotkey-row">
              <kbd class="hotkey-display" aria-label={hotkeyDisplay}>{hotkeyDisplay}</kbd>
              <button
                type="button"
                class="secondary"
                class:hotkey-recording={recordingHotkey}
                onclick={startHotkeyRecording}
              >
                {recordingHotkey ? t("settings.hotkeyRecording") : t("settings.hotkeyChange")}
              </button>
              {#if globalShortcut !== DEFAULT_GLOBAL_SHORTCUT}
                <button
                  type="button"
                  class="secondary"
                  in:fade={hintFadeInParams}
                  out:fade={hintFadeOutParams}
                  onclick={resetHotkey}
                >
                  {t("settings.hotkeyReset")}
                </button>
              {/if}
            </div>
            {#if hotkeyError}
              <p
                class="settings-hint deps-error"
                in:fade={hintFadeInParams}
                out:fade={hintFadeOutParams}
              >
                {hotkeyError}
              </p>
            {/if}
          </div>
          <div class="settings-divider"></div>
        {/if}

        {#if onTokenStatsPeriodChange && onTokenStatsChange}
          <TokenSavingsPanel
            stats={tokenStats}
            period={tokenStatsPeriod}
            onPeriodChange={onTokenStatsPeriodChange}
            onStatsChange={onTokenStatsChange}
          />
          <div class="settings-divider"></div>
        {/if}

        <div class="settings-section">
          <div class="settings-section-title">{t("gatekeeper.title")}</div>
          <p class="settings-hint">{t("gatekeeper.hint")}</p>
          <div class="gatekeeper-actions">
            <button
              type="button"
              class="secondary gatekeeper-copy-btn"
              class:gatekeeper-copy-success={gatekeeperCopied}
              onclick={copyGatekeeperCommand}
            >
              {#key gatekeeperCopied}
                <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
                  {gatekeeperCopied ? t("gatekeeper.copied") : t("gatekeeper.copyCommand")}
                </span>
              {/key}
            </button>
            {#if gatekeeperCopyError}
              <p class="settings-hint deps-error" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
                {gatekeeperCopyError}
              </p>
            {/if}
            <button type="button" class="secondary" onclick={openPrivacySettings}>
              {t("gatekeeper.openSettings")}
            </button>
          </div>
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section">
          <div class="settings-section-title">{t("settings.finderTitle")}</div>
          <p class="settings-hint">{t("settings.finderHint")}</p>
          {#if finderActionInstalled}
            <p
              class="settings-hint settings-finder-status"
              in:fade={hintFadeInParams}
              out:fade={hintFadeOutParams}
            >
              {t("settings.finderInstalled")}
            </p>
          {:else if onInstallFinderAction}
            <button
              type="button"
              class="secondary settings-finder-install-btn"
              disabled={finderActionBusy}
              onclick={onInstallFinderAction}
            >
              {#key finderActionBusy}
                <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
                  {finderActionBusy ? t("settings.finderInstalling") : t("settings.finderInstall")}
                </span>
              {/key}
            </button>
          {/if}
          {#if finderActionNotice}
            <p
              class="settings-hint settings-finder-notice"
              in:fade={hintFadeInParams}
              out:fade={hintFadeOutParams}
            >
              {finderActionNotice}
            </p>
          {/if}
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section">
          <div class="settings-section-title">{t("update.settingsTitle")}</div>
          <p class="settings-hint">{t("update.settingsHint")}</p>
          {#if onCheckForUpdates}
            <button
              type="button"
              class="secondary"
              disabled={updateCheckBusy}
              onclick={onCheckForUpdates}
            >
              {updateCheckBusy ? t("update.checking") : t("update.checkButton")}
            </button>
          {/if}
          {#if updateStatusNote}
            <p
              class="settings-update-status"
              class:settings-update-status-ok={updateStatusOk}
              in:fade={hintFadeInParams}
              out:fade={hintFadeOutParams}
            >
              {updateStatusNote}
            </p>
          {/if}
        </div>
      </div>
    {:else}
      <div
        id="settings-panel-file-support"
        role="tabpanel"
        aria-labelledby="settings-tab-file-support"
        in:fade={tabFadeInParams}
        out:fade={tabFadeOutParams}
      >
        <div class="settings-section">
          <div class="settings-section-title">{t("settings.ocrLanguageTitle")}</div>
          <p class="settings-hint settings-hint--multiline">{t("settings.ocrLanguageHint")}</p>
          <OcrLanguageSelector
            value={ocrLanguage}
            disabled={!ocrEnabled}
            onChange={onOcrLanguageChange}
          />
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section">
          <div class="settings-section-title">{t("settings.workersTitle")}</div>
          <p class="settings-hint">{t("settings.workersHint")}</p>
          <WorkersSlider
            value={workers}
            label={t("settings.workersTitle")}
            onChange={onWorkersChange}
          />
        </div>

        <div class="settings-divider"></div>

        <div class="settings-section">
          <DependencyPreflight />
        </div>
      </div>
    {/if}
    {/key}

    <div class="settings-divider"></div>

    <div class="settings-section">
      <button type="button" class="settings-link-card" onclick={onOpenAbout}>
        <span class="settings-link-text">{t("settings.aboutTitle")}</span>
        <span class="settings-link-chevron" aria-hidden="true">›</span>
      </button>
    </div>
  </div>
</div>