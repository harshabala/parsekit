<script lang="ts">
  import { t } from "../lib/i18n";

  let { onClose }: { onClose: () => void } = $props();

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
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div class="about-panel" role="document">
    <div class="about-header" id="about-title">{t("app.name")}</div>
    <div class="about-version">v0.1.0</div>
    <div class="about-tagline">{t("about.tagline")}</div>

    <div class="about-divider"></div>

    <div class="about-section">
      <div class="about-section-title">{t("about.poweredBy")}</div>
      <div class="about-text">{t("about.poweredByValue")}</div>
    </div>

    <div class="about-section">
      <div class="about-section-title">{t("about.license")}</div>
      <div class="about-text">{t("about.licenseValue")}</div>
      <div class="about-text-small">{t("about.licenseNote")}</div>
    </div>

    <button class="about-close-btn" onclick={onClose}>{t("about.close")}</button>
  </div>
</div>
