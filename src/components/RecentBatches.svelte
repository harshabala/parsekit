<script lang="ts">
  import { locale, t } from "../lib/i18n";
  import type { BatchResult } from "../lib/types";

  let { batches, onOpenFolder }: { batches: BatchResult[]; onOpenFolder: (path: string) => void } = $props();

  function formatDate(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      const localeTag = locale === "zh" ? "zh-Hans" : locale;
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
</script>

{#if batches.length > 0}
  <div class="section">
    <div class="section-title">{t("recent.title")}</div>
    <div class="card" style="padding: 0; overflow: hidden;">
      {#each batches as batch}
        <div class="history-item">
          <div class="history-top">
            <div class="history-info">
              <span class="history-name">{batch.inputDir.split("/").pop() || t("recent.batch")}</span>
              <span class="history-meta">
                {t("recent.meta", {
                  date: formatDate(batch.timestamp),
                  count: batch.fileCount,
                  format: batch.format.toUpperCase(),
                })}
              </span>
            </div>
            <button class="secondary history-open-btn" onclick={() => onOpenFolder(batch.outputDir)}>
              {t("recent.open")}
            </button>
          </div>
          {#if batch.errors > 0}
            <span class="history-meta" style="color: #ff3b30;">
              {batch.errors === 1
                ? t("recent.errorsOne")
                : t("recent.errors", { count: batch.errors })}
            </span>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
