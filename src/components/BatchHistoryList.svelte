<script lang="ts">
  import { getLocale, t } from "../lib/i18n.svelte";
  import type { BatchResult } from "../lib/types";

  let {
    batches,
    onOpenFolder,
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
  } = $props();

  function formatDate(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      const loc = getLocale();
      const localeTag = loc === "zh" ? "zh-Hans" : loc;
      return d.toLocaleDateString(localeTag, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return timestamp;
    }
  }

  function batchLabel(batch: BatchResult): string {
    const name = batch.inputDir.split("/").pop();
    return name || t("recent.batch");
  }
</script>

<div class="card history-card">
  {#each batches as batch (batch.id)}
    <div class="history-item">
      <div class="history-top">
        <div class="history-info">
          <span class="history-name">{batchLabel(batch)}</span>
          <span class="history-meta">
            {t("recent.meta", {
              date: formatDate(batch.timestamp),
              count: batch.fileCount,
              format: batch.format.toUpperCase(),
            })}
          </span>
        </div>
        <button
          type="button"
          class="secondary history-open-btn"
          onclick={() => onOpenFolder(batch.outputDir)}
        >
          {t("recent.open")}
        </button>
      </div>
      {#if batch.errors > 0}
        <span class="history-meta history-meta-error">
          {batch.errors === 1
            ? t("recent.errorsOne")
            : t("recent.errors", { count: batch.errors })}
        </span>
      {/if}
    </div>
  {/each}
</div>