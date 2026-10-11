<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import { focusTrap } from "../lib/focusTrap";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import type { BatchResult } from "../lib/types";
  import BatchHistoryList from "./BatchHistoryList.svelte";

  let {
    batches,
    onOpenFolder,
    onRerun,
    appVersion = "0.0.0",
    onClose,
  }: {
    batches: BatchResult[];
    onOpenFolder: (path: string) => void;
    onRerun: (batch: BatchResult) => void;
    appVersion?: string;
    onClose: (options?: { instant?: boolean }) => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose({ instant: true });
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="settings-screen"
  role="dialog"
  aria-modal="true"
  aria-labelledby="history-title"
  use:focusTrap={{ restoreFocus: false }}
>
  <div class="settings-header">
    <button type="button" class="settings-back-btn" onclick={() => onClose()}>{t("history.back")}</button>
    <span class="settings-header-title" id="history-title">{t("history.title")}</span>
  </div>

  <div class="settings-scroll">
    {#if batches.length === 0}
      <div
        class="history-empty"
        in:fade={hintFadeInParams}
        out:fade={hintFadeOutParams}
      >
        <p class="settings-hint">{t("history.empty")}</p>
        <button type="button" class="secondary history-empty-action" onclick={() => onClose()}>
          {t("history.backToConvert")}
        </button>
      </div>
    {:else}
      <div in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
        <p class="settings-hint">{t("history.hint", { count: batches.length })}</p>
        <BatchHistoryList {batches} {onOpenFolder} {onRerun} {appVersion} />
      </div>
    {/if}
  </div>
</div>