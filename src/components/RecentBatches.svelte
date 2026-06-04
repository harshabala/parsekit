<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { BatchResult } from "../lib/types";
  import BatchHistoryList from "./BatchHistoryList.svelte";

  let {
    latestBatch,
    showHistoryButton = false,
    onOpenFolder,
    onOpenHistory,
  }: {
    latestBatch: BatchResult | null;
    showHistoryButton?: boolean;
    onOpenFolder: (path: string) => void;
    onOpenHistory: () => void;
  } = $props();
</script>

{#if latestBatch}
  <div class="section">
    <div class="section-header-row">
      <div class="section-title">{t("recent.title")}</div>
      {#if showHistoryButton}
        <button
          type="button"
          class="icon-btn section-history-btn"
          onclick={onOpenHistory}
          title={t("recent.viewHistory")}
          aria-label={t("recent.viewHistory")}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M3 3v5h5M12 7v5l4 2"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      {/if}
    </div>
    <BatchHistoryList batches={[latestBatch]} {onOpenFolder} />
  </div>
{/if}