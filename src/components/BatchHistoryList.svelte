<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { invoke } from "@tauri-apps/api/core";
  import { buildErrorReport } from "../lib/errorReport";
  import { getLocale, t } from "../lib/i18n.svelte";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import type { BatchResult } from "../lib/types";

  let {
    batches,
    onOpenFolder,
    onRerun,
    appVersion = "0.0.0",
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
    onRerun?: (batch: BatchResult) => void;
    appVersion?: string;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

  let copiedBatchId = $state<string | null>(null);
  let copyError = $state<string | null>(null);

  function canRerun(batch: BatchResult): boolean {
    if (!onRerun) return false;
    const selectedLabel = t("recent.selectedFiles");
    return (
      (batch.sourcePaths?.length ?? 0) > 0 || (!!batch.inputDir && batch.inputDir !== selectedLabel)
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

  function successLabel(batch: BatchResult): string {
    if (batch.parsed === 1) {
      return t("history.successOne");
    }
    return t("history.success", { count: batch.parsed });
  }

  async function copyErrors(batch: BatchResult) {
    if (!batch.fileErrors?.length) return;
    copyError = null;
    try {
      await invoke("copy_text_to_clipboard", {
        text: buildErrorReport(batch, appVersion),
      });
      try {
        await invoke("trigger_haptic");
      } catch {
        /* optional */
      }
      copiedBatchId = batch.id;
      setTimeout(() => {
        if (copiedBatchId === batch.id) copiedBatchId = null;
      }, 2800);
    } catch (e) {
      copiedBatchId = null;
      copyError = e instanceof Error ? e.message : String(e) || t("history.copyFailed");
    }
  }
</script>

<div class="card history-card">
  {#each batches as batch (batch.id)}
    <div class="history-item">
      <div class="history-primary">
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

      <div class="history-footer">
        {#if batch.errors > 0}
          <div class="history-status history-status-error">
            <span class="history-status-text">
              {batch.errors === 1
                ? t("recent.errorsOne")
                : t("recent.errors", { count: batch.errors })}
            </span>
            {#if batch.fileErrors?.length}
              <button
                type="button"
                class="secondary history-copy-btn"
                class:history-copy-success={copiedBatchId === batch.id}
                title={copiedBatchId === batch.id ? t("history.copyConfirm") : undefined}
                aria-label={copiedBatchId === batch.id
                  ? t("history.copyConfirm")
                  : t("history.copyErrors")}
                onclick={() => copyErrors(batch)}
              >
                {#key copiedBatchId === batch.id}
                  <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
                    {copiedBatchId === batch.id
                      ? t("history.errorsCopied")
                      : t("history.copyErrors")}
                  </span>
                {/key}
              </button>
            {/if}
          </div>
        {:else}
          <div class="history-status history-status-success">
            <span class="history-status-text">{successLabel(batch)}</span>
          </div>
        {/if}

        <div class="history-actions">
          {#if canRerun(batch)}
            <button
              type="button"
              class="secondary history-action-btn"
              onclick={() => onRerun?.(batch)}
            >
              {t("history.rerun")}
            </button>
          {/if}
          <button
            type="button"
            class="secondary history-action-btn"
            onclick={() => onOpenFolder(batch.outputDir)}
          >
            {t("recent.open")}
          </button>
        </div>
      </div>
    </div>
  {/each}
</div>

{#if copyError}
  <p
    class="history-copy-error settings-hint deps-error"
    role="alert"
    in:fade={hintFadeInParams}
    out:fade={hintFadeOutParams}
  >
    {copyError}
  </p>
{/if}
