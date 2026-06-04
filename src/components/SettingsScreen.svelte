<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { AppLocale } from "../lib/i18n.svelte";
  import { t } from "../lib/i18n.svelte";
  import type { OcrLanguageCode } from "../lib/ocrLanguages";
  import type { ThemeMode } from "../lib/types";
  import LanguageSelector from "./LanguageSelector.svelte";
  import OcrLanguageSelector from "./OcrLanguageSelector.svelte";
  import ThemeSelector from "./ThemeSelector.svelte";

  const REPO_URL = "https://github.com/harshabala/parsedock";

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
    onClose: () => void;
  } = $props();

  let version = $state("0.2.0");

  onMount(async () => {
    try {
      const info = await invoke<{ version?: string }>("get_system_info");
      if (info.version) version = info.version;
    } catch {
      /* keep default */
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
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
      <div class="workers-row">
        <input
          type="range"
          min="1"
          max="16"
          step="1"
          value={workers}
          aria-label={t("settings.workersTitle")}
          oninput={(e) => onWorkersChange(Number((e.currentTarget as HTMLInputElement).value))}
        />
        <span class="workers-value">{workers}</span>
      </div>
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

    <div class="settings-section settings-about">
      <div class="settings-section-title">{t("settings.aboutTitle")}</div>
      <p class="settings-about-version">v{version}</p>
      <p class="settings-hint">{t("settings.aboutDescription")}</p>
      <p class="settings-hint">{t("settings.aboutTagline")}</p>
      <p class="settings-hint">{t("settings.aboutFormats")}</p>
      <div class="settings-about-meta">
        <span class="settings-meta-label">{t("settings.aboutPoweredBy")}</span>
        <span>{t("settings.aboutPoweredByValue")}</span>
      </div>
      <div class="settings-about-meta">
        <span class="settings-meta-label">{t("settings.aboutLicense")}</span>
        <span>{t("settings.aboutLicenseValue")}</span>
      </div>
      <p class="settings-hint settings-license-note">{t("settings.aboutLicenseNote")}</p>
      <a class="settings-repo-link" href={REPO_URL} target="_blank" rel="noopener noreferrer">
        {t("settings.aboutRepository")}
      </a>
    </div>
  </div>
</div>