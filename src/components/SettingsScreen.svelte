<script lang="ts">
  import type { AppLocale } from "../lib/i18n";
  import { t } from "../lib/i18n";
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
    onLocaleChange,
    onOcrLanguageChange,
    onThemeChange,
    onClose,
  }: {
    locale: AppLocale;
    ocrLanguage: OcrLanguageCode;
    ocrEnabled: boolean;
    theme: ThemeMode;
    onLocaleChange: (code: AppLocale) => void;
    onOcrLanguageChange: (code: OcrLanguageCode) => void;
    onThemeChange: (mode: ThemeMode) => void;
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
      <div class="about-section-title">{t("settings.appearanceTitle")}</div>
      <p class="settings-hint">{t("settings.appearanceHint")}</p>
      <ThemeSelector value={theme} onChange={onThemeChange} />
    </div>

    <button type="button" class="about-close-btn" onclick={onClose}>{t("settings.close")}</button>
  </div>
</div>