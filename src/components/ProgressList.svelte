<script lang="ts">
  import type { FileProgress } from "../lib/types";

  let { files, total, isParsing }: { files: FileProgress[]; total: number; isParsing: boolean } = $props();

  let completedCount = $derived(files.filter(f => f.status === "done").length);
  let errorCount = $derived(files.filter(f => f.status === "error").length);
  let skippedCount = $derived(files.filter(f => f.status === "skipped").length);
  let progressPercent = $derived(total > 0 ? Math.round((completedCount + errorCount + skippedCount) / total * 100) : 0);

  function statusIcon(status: string): string {
    switch (status) {
      case "parsing": return "\u27F3";
      case "done": return "\u2713";
      case "error": return "\u2717";
      case "skipped": return "\u2014";
      default: return "\u00B7";
    }
  }
</script>

<div class="section">
  <div class="section-title">Processing</div>
  <div class="card progress-container">
    <div class="row">
      <span>{isParsing ? "Parsing Documents..." : "Parsing Complete"}</span>
      <span>{progressPercent}%</span>
    </div>
    <div class="progress-bar">
      <div class="progress-fill" style="width: {progressPercent}%"></div>
    </div>

    {#if !isParsing && completedCount + errorCount > 0}
      <div class="progress-summary">
        {completedCount} parsed, {errorCount} errors out of {total} files
      </div>
    {/if}

    <div class="file-list">
      {#each files as file}
        <div class="file-item">
          <span class="file-name" title={file.name}>
            {file.name}
          </span>
          <span class="status-icon status-{file.status}">
            {statusIcon(file.status)}
          </span>
        </div>
      {/each}
    </div>
  </div>
</div>
