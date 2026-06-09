<script lang="ts">
  import { getLocale, t } from "../lib/i18n.svelte";
  import type { BatchResult } from "../lib/types";

  let {
    batches,
    onOpenFolder,
    onRerun,
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
    onRerun?: (batch: BatchResult) => void;
  } = $props();

  function canRerun(batch: BatchResult): boolean {
    if (!onRerun) return false;
    const selectedLabel = t("recent.selectedFiles");
    return (
      (batch.sourcePaths?.length ?? 0) > 0 ||
      (!!batch.inputDir && batch.inputDir !== selectedLabel)
    );
  }

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
            {batch.fileCount === 1
              ? t("recent.metaOne", {
                  date: formatDate(batch.timestamp),
                  format: batch.format.toUpperCase(),
                })
              : t("recent.meta", {
                  date: formatDate(batch.timestamp),
                  count: batch.fileCount,
                  format: batch.format.toUpperCase(),
                })}
          </span>
        </div>
        <div class="history-actions">
          {#if canRerun(batch)}
            <button
              type="button"
              class="secondary history-rerun-btn"
              onclick={() => onRerun?.(batch)}
            >
              {t("history.rerun")}
            </button>
          {/if}
          <button
            type="button"
            class="secondary history-open-btn"
            onclick={() => onOpenFolder(batch.outputDir)}
          >
            {t("recent.open")}
          </button>
        </div>
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