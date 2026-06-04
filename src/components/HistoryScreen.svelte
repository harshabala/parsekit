<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { BatchResult } from "../lib/types";
  import BatchHistoryList from "./BatchHistoryList.svelte";

  let {
    batches,
    onOpenFolder,
    onClose,
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
    onClose: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-screen" role="dialog" aria-modal="true" aria-labelledby="history-title">
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={onClose}>{t("history.back")}</button>
    <span class="settings-header-title" id="history-title">{t("history.title")}</span>
  </div>

  <div class="settings-scroll">
    {#if batches.length === 0}
      <p class="settings-hint">{t("history.empty")}</p>
    {:else}
      <p class="settings-hint">{t("history.hint", { count: batches.length })}</p>
      <BatchHistoryList {batches} {onOpenFolder} />
    {/if}
  </div>
</div>