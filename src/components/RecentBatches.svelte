<script lang="ts">
  import type { BatchResult } from "../lib/types";

  let { batches, onOpenFolder }: { batches: BatchResult[]; onOpenFolder: (path: string) => void } = $props();

  function formatDate(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      return d.toLocaleDateString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
    } catch {
      return timestamp;
    }
  }
</script>

{#if batches.length > 0}
  <div class="section">
    <div class="section-title">Recent Batches</div>
    <div class="card" style="padding: 0; overflow: hidden;">
      {#each batches as batch}
        <div class="history-item">
          <div class="history-top">
            <div class="history-info">
              <span class="history-name">{batch.inputDir.split("/").pop() || "Batch"}</span>
              <span class="history-meta">
                {formatDate(batch.timestamp)} &middot; {batch.fileCount} files &middot; {batch.format.toUpperCase()}
              </span>
            </div>
            <button class="secondary history-open-btn" onclick={() => onOpenFolder(batch.outputDir)}>
              Open
            </button>
          </div>
          {#if batch.errors > 0}
            <span class="history-meta" style="color: #ff3b30;">{batch.errors} error{batch.errors > 1 ? "s" : ""}</span>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
