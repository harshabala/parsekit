<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "../lib/i18n.svelte";

  const REPO_URL = "https://github.com/harshabala/parsedock";

  let { onClose }: { onClose: () => void } = $props();

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

<div class="settings-screen" role="dialog" aria-modal="true" aria-labelledby="about-title">
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={onClose}>{t("about.back")}</button>
    <span class="settings-header-title" id="about-title">{t("settings.aboutTitle")}</span>
  </div>

  <div class="settings-scroll about-scroll">
    <p class="about-version">v{version}</p>
    <p class="about-lead">{t("settings.aboutDescription")}</p>
    <p class="about-tagline">{t("settings.aboutTagline")}</p>
    <p class="settings-hint about-privacy about-wrap">{t("settings.aboutPrivacy")}</p>

    <div class="settings-divider"></div>

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