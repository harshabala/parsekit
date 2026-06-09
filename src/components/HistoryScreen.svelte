<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import type { BatchResult } from "../lib/types";
  import BatchHistoryList from "./BatchHistoryList.svelte";

  let {
    batches,
    onOpenFolder,
    onRerun,
    onSaveErrors,
    onClose,
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
    onRerun: (batch: BatchResult) => void;
    onSaveErrors?: (batch: BatchResult) => void;
    onClose: () => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

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
      <p class="settings-hint" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
        {t("history.empty")}
      </p>
    {:else}
      <div in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
        <p class="settings-hint">{t("history.hint", { count: batches.length })}</p>
        <BatchHistoryList {batches} {onOpenFolder} {onRerun} {onSaveErrors} />
      </div>
    {/if}
  </div>
</div>