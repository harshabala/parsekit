<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import ClockCounterClockwiseIcon from "phosphor-svelte/lib/ClockCounterClockwiseIcon";
  import { hintFadeIn, hintFadeOut, sectionFlyIn, sectionFlyOut } from "../lib/motion";
  import { t } from "../lib/i18n.svelte";
  import type { BatchResult } from "../lib/types";
  import BatchHistoryList from "./BatchHistoryList.svelte";

  let {
    latestBatch,
    showHistoryButton = false,
    appVersion = "0.0.0",
    onOpenFolder,
    onOpenHistory,
  }: {
    latestBatch: BatchResult | null;
    showHistoryButton?: boolean;
    appVersion?: string;
    onOpenFolder: (path: string) => void;
    onOpenHistory: () => void;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const sectionFlyInParams = $derived(sectionFlyIn(reducedMotion));
  const sectionFlyOutParams = $derived(sectionFlyOut(reducedMotion));
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));
</script>

{#if latestBatch}
  <div class="section" in:fly={sectionFlyInParams} out:fly={sectionFlyOutParams}>
    <div class="section-header-row">
      <div class="section-title">{t("recent.title")}</div>
      {#if showHistoryButton}
        <button
          type="button"
          class="icon-btn section-history-btn"
          in:fade={hintFadeInParams}
          out:fade={hintFadeOutParams}
          onclick={onOpenHistory}
          title={t("recent.viewHistory")}
          aria-label={t("recent.viewHistory")}
        >
          <ClockCounterClockwiseIcon size={16} weight="regular" aria-hidden="true" />
        </button>
      {/if}
    </div>
    <BatchHistoryList batches={[latestBatch]} {onOpenFolder} {appVersion} />
  </div>
{/if}