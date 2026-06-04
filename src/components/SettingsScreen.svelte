<script lang="ts">
  import type { AppLocale } from "../lib/i18n.svelte";
  import { t } from "../lib/i18n.svelte";
  import type { OcrLanguageCode } from "../lib/ocrLanguages";
  import type { ThemeMode } from "../lib/types";
  import LanguageSelector from "./LanguageSelector.svelte";
  import OcrLanguageSelector from "./OcrLanguageSelector.svelte";
  import ThemeSelector from "./ThemeSelector.svelte";
  import WorkersSlider from "./WorkersSlider.svelte";
  import DependencyPreflight from "./DependencyPreflight.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let {
    locale: localeValue,
    ocrLanguage,
    ocrEnabled,
    theme,
    workers,
    launchAtLogin,
    onLocaleChange,
    onOcrLanguageChange,
    onThemeChange,
    onWorkersChange,
    onLaunchAtLoginChange,
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
    onClose,
  }: {
    locale: AppLocale;
    ocrLanguage: OcrLanguageCode;
    ocrEnabled: boolean;
    theme: ThemeMode;
    workers: number;
    launchAtLogin: boolean;
    onLocaleChange: (code: AppLocale) => void;
    onOcrLanguageChange: (code: OcrLanguageCode) => void;
    onThemeChange: (mode: ThemeMode) => void;
    onWorkersChange: (value: number) => void;
    onLaunchAtLoginChange: (enabled: boolean) => void;
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
    onClose: () => void;
  } = $props();

  let gatekeeperCopied = $state(false);
  let gatekeeperCopyError = $state<string | null>(null);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
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

  <div class="settings-scroll">
    <div class="settings-section">
      <div class="settings-section-title">{t("settings.appLanguageTitle")}</div>
      <p class="settings-hint">{t("settings.appLanguageHint")}</p>
      <LanguageSelector value={localeValue} onChange={onLocaleChange} />
    </div>

    <div class="settings-divider"></div>

    <div class="settings-section">
      <div class="settings-section-title">{t("settings.ocrLanguageTitle")}</div>
      <p class="settings-hint">{t("settings.ocrLanguageHint")}</p>
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

    <div class="settings-section">
      <DependencyPreflight />
    </div>

    <div class="settings-divider"></div>

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
          {gatekeeperCopied ? t("gatekeeper.copied") : t("gatekeeper.copyCommand")}
        </button>
        {#if gatekeeperCopyError}
          <p class="settings-hint deps-error">{gatekeeperCopyError}</p>
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
        <p class="settings-hint settings-finder-status">{t("settings.finderInstalled")}</p>
      {:else if onInstallFinderAction}
        <button
          type="button"
          class="secondary settings-finder-install-btn"
          disabled={finderActionBusy}
          onclick={onInstallFinderAction}
        >
          {finderActionBusy ? t("settings.finderInstalling") : t("settings.finderInstall")}
        </button>
      {/if}
      {#if finderActionNotice}
        <p class="settings-hint settings-finder-notice">{finderActionNotice}</p>
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
        >
          {updateStatusNote}
        </p>
      {/if}
    </div>

    <div class="settings-divider"></div>

    <div class="settings-section">
      <button type="button" class="settings-link-card" onclick={onOpenAbout}>
        <span class="settings-link-text">{t("settings.aboutTitle")}</span>
        <span class="settings-link-chevron" aria-hidden="true">›</span>
      </button>
    </div>
  </div>
</div>