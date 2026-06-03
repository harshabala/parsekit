<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "../lib/i18n.svelte";

  const REPO_URL = "https://github.com/harshabala/parsedock";

  let { onClose }: { onClose: () => void } = $props();
  let version = $state("0.1.0");

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
    <div class="about-version">v{version}</div>
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

    <div class="about-section">
      <a class="about-link" href={REPO_URL} target="_blank" rel="noopener noreferrer">
        {t("about.repository")}
      </a>
    </div>

    <button type="button" class="about-close-btn" onclick={onClose}>{t("about.close")}</button>
  </div>
</div>