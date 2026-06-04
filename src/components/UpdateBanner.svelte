<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { UpdateInfo } from "../lib/update";

  let {
    info,
    installing = false,
    error = null,
    onInstall,
    onDismiss,
  }: {
    info: UpdateInfo;
    installing?: boolean;
    error?: string | null;
    onInstall: () => void;
    onDismiss: () => void;
  } = $props();

  const versionLabel = $derived(info.version ?? "?");
</script>

<div
  class="update-banner"
  role="status"
  aria-live="polite"
  aria-label={t("update.bannerAria", { version: versionLabel })}
>
  <p class="update-banner-text">
    {t("update.available", { version: versionLabel })}
  </p>
  <div class="update-banner-actions">
    <button
      type="button"
      class="update-banner-primary"
      disabled={installing}
      onclick={onInstall}
    >
      {installing ? t("update.installing") : t("update.installRestart")}
    </button>
    <button
      type="button"
      class="update-banner-secondary"
      disabled={installing}
      onclick={onDismiss}
    >
      {t("update.later")}
    </button>
  </div>
  {#if error}
    <p class="update-banner-error">{error}</p>
  {/if}
</div>