<script lang="ts">
  import type { AppLocale } from "../lib/i18n.svelte";
  import { t } from "../lib/i18n.svelte";
  import type { OcrLanguageCode } from "../lib/ocrLanguages";
  import type { ThemeMode } from "../lib/types";
  import LanguageSelector from "./LanguageSelector.svelte";
  import OcrLanguageSelector from "./OcrLanguageSelector.svelte";
  import ThemeSelector from "./ThemeSelector.svelte";

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleOverlayKeydown(e: KeyboardEvent) {
    if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="about-overlay"
  onclick={handleOverlayClick}
  onkeydown={handleOverlayKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-title"
  tabindex="-1"
>
  <div class="about-panel settings-panel" role="document">
    <div class="about-header" id="settings-title">{t("settings.title")}</div>

    <div class="about-divider"></div>

    <div class="settings-section">
      <div class="about-section-title">{t("settings.appLanguageTitle")}</div>
      <p class="settings-hint">{t("settings.appLanguageHint")}</p>
      <LanguageSelector value={localeValue} onChange={onLocaleChange} />
    </div>

    <div class="about-divider"></div>

    <div class="settings-section">
      <div class="about-section-title">{t("settings.ocrLanguageTitle")}</div>
      <p class="settings-hint">{t("settings.ocrLanguageHint")}</p>
      <OcrLanguageSelector
        value={ocrLanguage}
        disabled={!ocrEnabled}
        onChange={onOcrLanguageChange}
      />
    </div>

    <div class="about-divider"></div>

    <div class="settings-section">
      <div class="about-section-title">{t("settings.workersTitle")}</div>
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

    <div class="about-divider"></div>

    <div class="settings-section">
      <div class="about-section-title">{t("settings.appearanceTitle")}</div>
      <p class="settings-hint">{t("settings.appearanceHint")}</p>
      <ThemeSelector value={theme} onChange={onThemeChange} />
    </div>

    <div class="about-divider"></div>

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

    <button type="button" class="about-close-btn" onclick={onClose}>{t("settings.close")}</button>
  </div>
</div>