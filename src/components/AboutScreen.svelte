<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Command } from "@tauri-apps/plugin-shell";
  import { t } from "../lib/i18n.svelte";
  import { focusTrap } from "../lib/focusTrap";

  const AUTHOR_URL = "https://github.com/harshabala";

  let {
    onClose,
  }: {
    onClose: (options?: { instant?: boolean }) => void;
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
      // Nested About subview: Escape steps back to Settings without animating the outer panel.
      onClose({ instant: true });
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

<div
  class="settings-screen"
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  use:focusTrap={{ restoreFocus: false }}
>
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={() => onClose()}>{t("about.back")}</button>
    <span class="settings-header-title" id="about-title">{t("about.title")}</span>
  </div>

  <div class="settings-scroll about-scroll selectable-content">
    <p class="about-version" aria-label={t("about.versionLabel", { version })}>v{version}</p>

    <p class="about-hero">{t("about.heroLead")}</p>
    <p class="about-local">{t("about.localLine")}</p>

    <h2 class="about-section-title">{t("about.benefitsTitle")}</h2>
    <ul class="about-list about-list-compact">
      <li>{t("about.benefit1")}</li>
      <li>{t("about.benefit2")}</li>
      <li>{t("about.benefit4")}</li>
      <li>{t("about.benefit5")}</li>
    </ul>

    <h2 class="about-section-title">{t("about.formatsTitle")}</h2>
    <div class="about-formats">
      <div class="about-formats-col">
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

    <p class="settings-hint about-powered">{t("about.poweredBy")} · {t("about.licenseLine")}</p>

    <p class="about-attribution">
      {t("about.craftedBy")}
      <button type="button" class="about-author-link" onclick={openAuthorProfile}>
        {t("about.craftedByName")}
      </button>
    </p>
  </div>
</div>