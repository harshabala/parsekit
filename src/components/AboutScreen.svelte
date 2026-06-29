<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Command } from "@tauri-apps/plugin-shell";
  import { t } from "../lib/i18n.svelte";

  const AUTHOR_URL = "https://github.com/harshabala";

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

  async function openAuthorProfile() {
    try {
      await Command.create("open", [AUTHOR_URL]).spawn();
    } catch {
      window.open(AUTHOR_URL, "_blank", "noopener,noreferrer");
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-screen" role="dialog" aria-modal="true" aria-labelledby="about-title">
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={onClose}>{t("about.back")}</button>
    <span class="settings-header-title" id="about-title">{t("about.title")}</span>
  </div>

  <div class="settings-scroll about-scroll selectable-content">
    <p class="about-version" aria-label={t("about.versionLabel", { version })}>v{version}</p>

    <p class="about-hero">{t("about.heroLead")}</p>
    <p class="settings-hint about-body">{t("about.heroBody")}</p>
    <p class="settings-hint about-body">{t("about.heroValue")}</p>
    <p class="about-local about-emphasis">{t("about.localLine")}</p>
    <p class="about-local about-emphasis">{t("about.privacyLine")}</p>

    <div class="settings-divider"></div>

    <h2 class="about-section-title">{t("about.benefitsTitle")}</h2>
    <ul class="about-list">
      <li>{t("about.benefit1")}</li>
      <li>{t("about.benefit2")}</li>
      <li>{t("about.benefit3")}</li>
      <li>{t("about.benefit4")}</li>
      <li>{t("about.benefit5")}</li>
      <li>{t("about.benefit6")}</li>
      <li>{t("about.benefit7")}</li>
    </ul>

    <div class="settings-divider"></div>

    <h2 class="about-section-title">{t("about.formatsTitle")}</h2>
    <div class="about-formats">
      <div class="about-formats-col">
        <p class="about-formats-label">{t("about.convertLabel")}</p>
        <ul class="about-list about-list-compact">
          <li>{t("about.formatPdf")}</li>
          <li>{t("about.formatWord")}</li>
          <li>{t("about.formatExcel")}</li>
          <li>{t("about.formatPowerPoint")}</li>
          <li>{t("about.formatImages")}</li>
        </ul>
      </div>
      <p class="about-formats-into" aria-hidden="true">{t("about.intoLabel")}</p>
      <div class="about-formats-col">
        <ul class="about-list about-list-compact about-formats-output">
          <li>{t("about.outputMarkdown")}</li>
          <li>{t("about.outputText")}</li>
          <li>{t("about.outputJson")}</li>
        </ul>
      </div>
    </div>

    <div class="settings-divider"></div>

    <p class="settings-hint about-powered">{t("about.poweredBy")}</p>
    <p class="settings-hint about-license">{t("about.licenseLine")}</p>

    <div class="settings-divider"></div>

    <p class="about-attribution">
      {t("about.craftedBy")}
      <button type="button" class="about-author-link" onclick={openAuthorProfile}>
        {t("about.craftedByName")}
      </button>
    </p>
  </div>
</div>