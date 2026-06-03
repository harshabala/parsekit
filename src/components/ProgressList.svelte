<script lang="ts">
  import { t } from "../lib/i18n";
  import type { FileProgress } from "../lib/types";

  let { files, total, isParsing }: { files: FileProgress[]; total: number; isParsing: boolean } = $props();

  let completedCount = $derived(files.filter((f) => f.status === "done").length);
  let errorCount = $derived(files.filter((f) => f.status === "error").length);
  let skippedCount = $derived(files.filter((f) => f.status === "skipped").length);
  let parsingCount = $derived(files.filter((f) => f.status === "parsing").length);
  let pendingCount = $derived(files.filter((f) => f.status === "pending").length);

  let finishedCount = $derived(completedCount + errorCount + skippedCount);
  let progressPercent = $derived(
    total > 0 ? Math.round((finishedCount / total) * 100) : 0
  );

  function statusIcon(status: string): string {
    switch (status) {
      case "parsing":
        return "\u27F3";
      case "done":
        return "\u2713";
      case "error":
        return "\u2717";
      case "skipped":
        return "\u2014";
      default:
        return "\u00B7";
    }
  }
</script>

<div class="section">
  <div class="section-title">{t("progress.title")}</div>
  <div class="card progress-container">
    <div class="row progress-header">
      <span>{isParsing ? t("progress.parsing") : t("progress.complete")}</span>
      <span class="progress-fraction" aria-live="polite">
        {t("progress.ofTotal", { done: finishedCount, total })}
      </span>
    </div>

    <div class="progress-bar" role="progressbar" aria-valuenow={progressPercent} aria-valuemin={0} aria-valuemax={100}>
      <div class="progress-fill" style="width: {progressPercent}%"></div>
    </div>
    <div class="progress-percent">{progressPercent}%</div>

    <div class="progress-stats" aria-live="polite">
      <span class="progress-stat progress-stat-done">
        {t("progress.completed", { count: completedCount })}
      </span>
      <span class="progress-stat progress-stat-active">
        {t("progress.inProgress", { count: parsingCount })}
      </span>
      <span class="progress-stat progress-stat-wait">
        {t("progress.waiting", { count: pendingCount })}
      </span>
      {#if errorCount > 0 || skippedCount > 0}
        <span class="progress-stat progress-stat-err">
          {#if errorCount > 0}
            {t("progress.failed", { count: errorCount })}
          {/if}
          {#if skippedCount > 0}
            {errorCount > 0 ? " · " : ""}{t("progress.skipped", { count: skippedCount })}
          {/if}
        </span>
      {/if}
    </div>

    {#if finishedCount > 0}
      <div class="progress-summary">
        {t("progress.summary", {
          parsed: completedCount,
          errors: errorCount,
          total,
        })}
      </div>
    {/if}

    <div class="file-list">
      {#each files as file (file.name)}
        <div class="file-item" class:file-item-active={file.status === "parsing"}>
          <span class="file-name" title={file.name}>{file.name}</span>
          <span class="status-icon status-{file.status}" title={file.status}>
            {statusIcon(file.status)}
          </span>
        </div>
      {/each}
    </div>
  </div>
</div>