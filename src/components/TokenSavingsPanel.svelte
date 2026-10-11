<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { t } from "../lib/i18n.svelte";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
  import {
    approximateChatGptMessages,
    formatTokenCount,
    resetTokenStats,
    type TokenStats,
  } from "../lib/tokenStats";
  import type { TokenStatsPeriod } from "../lib/store";

  let {
    stats,
    period,
    onPeriodChange,
    onStatsChange,
    compact = false,
  }: {
    stats: TokenStats | null;
    period: TokenStatsPeriod;
    onPeriodChange: (period: TokenStatsPeriod) => void;
    onStatsChange: (stats: TokenStats) => void;
    /** Hide outer title/long copy when the parent section already labels this block. */
    compact?: boolean;
  } = $props();

  const reducedMotion = $derived(prefersReducedMotion.current);
  const hintFadeInParams = $derived(hintFadeIn(reducedMotion));
  const hintFadeOutParams = $derived(hintFadeOut(reducedMotion));

  let resetting = $state(false);
  let resetError = $state<string | null>(null);

  const lifetimeTokens = $derived(stats?.total_tokens_saved ?? 0);
  const chatGptApprox = $derived(approximateChatGptMessages(lifetimeTokens));
  const hasData = $derived((stats?.total_files_converted ?? 0) > 0);

  const sortedFileTypes = $derived(
    stats
      ? Object.entries(stats.by_file_type).sort(([a], [b]) => a.localeCompare(b))
      : [],
  );

  function fileTypeLabel(type: string): string {
    const key = `tokenSavings.fileType.${type}`;
    const translated = t(key);
    return translated === key ? type.toUpperCase() : translated;
  }

  async function handleReset() {
    if (resetting) return;
    if (!confirm(t("tokenSavings.resetConfirm"))) return;
    resetting = true;
    resetError = null;
    try {
      const next = await resetTokenStats();
      onStatsChange(next);
    } catch (e) {
      resetError = e instanceof Error ? e.message : String(e);
    } finally {
      resetting = false;
    }
  }
</script>

<div class="token-savings-panel" class:token-savings-panel--compact={compact}>
  {#if !compact}
    <div class="settings-section-title">{t("tokenSavings.panelTitle")}</div>
    <p class="settings-hint">{t("tokenSavings.panelHint")}</p>
  {:else}
    <p class="settings-hint">{t("tokenSavings.panelHintShort")}</p>
  {/if}

  <div class="token-savings-period" role="group" aria-label={t("tokenSavings.periodLabel")}>
    <div class="segmented-control token-savings-period-toggle">
      <button
        type="button"
        class="segment"
        class:active={period === "today"}
        aria-pressed={period === "today"}
        onclick={() => onPeriodChange("today")}
      >
        {t("tokenSavings.periodToday")}
      </button>
      <button
        type="button"
        class="segment"
        class:active={period === "month"}
        aria-pressed={period === "month"}
        onclick={() => onPeriodChange("month")}
      >
        {t("tokenSavings.periodMonth")}
      </button>
      <button
        type="button"
        class="segment"
        class:active={period === "lifetime"}
        aria-pressed={period === "lifetime"}
        onclick={() => onPeriodChange("lifetime")}
      >
        {t("tokenSavings.periodLifetime")}
      </button>
    </div>
    {#if !compact}
      <p class="settings-hint token-savings-period-hint">{t("tokenSavings.periodHint")}</p>
    {/if}
  </div>

  {#if !hasData}
    <p class="settings-hint token-savings-empty">{t("tokenSavings.empty")}</p>
  {:else}
    <dl class="token-savings-stats">
      <div class="token-savings-stat">
        <dt>{t("tokenSavings.lifetimeTotal")}</dt>
        <dd>{formatTokenCount(lifetimeTokens)}</dd>
      </div>
      <div class="token-savings-stat">
        <dt>{t("tokenSavings.filesConverted")}</dt>
        <dd>{formatTokenCount(stats?.total_files_converted ?? 0)}</dd>
      </div>
      {#if !compact || (stats?.total_pages_unlocked ?? 0) > 0}
        <div class="token-savings-stat token-savings-stat--pages">
          <dt>{t("tokenSavings.pagesUnlocked")}</dt>
          <dd>{formatTokenCount(stats?.total_pages_unlocked ?? 0)}</dd>
        </div>
      {/if}
      {#if !compact && (stats?.total_documents_unlocked ?? 0) > 0}
        <div class="token-savings-stat token-savings-stat--pages">
          <dt>{t("tokenSavings.documentsUnlocked")}</dt>
          <dd>{formatTokenCount(stats?.total_documents_unlocked ?? 0)}</dd>
        </div>
      {/if}
    </dl>

    {#if !compact && lifetimeTokens > 0}
      <p class="token-savings-approx">
        {t("tokenSavings.chatGptApprox", { count: formatTokenCount(chatGptApprox) })}
        <span class="token-savings-approx-label">{t("tokenSavings.approximateLabel")}</span>
      </p>
    {/if}

    {#if sortedFileTypes.length > 0}
      <div class="token-savings-by-type">
        {#if !compact}
          <div class="token-savings-by-type-title">{t("tokenSavings.byTypeTitle")}</div>
        {/if}
        <ul class="token-savings-by-type-list" class:token-savings-by-type-list--inline={compact}>
          {#each sortedFileTypes as [type, row]}
            <li>
              {#if compact}
                {fileTypeLabel(type)} {formatTokenCount(row.files)}
              {:else}
                {t("tokenSavings.byTypeRow", {
                  type: fileTypeLabel(type),
                  tokens: formatTokenCount(row.tokens_saved),
                  files: formatTokenCount(row.files),
                })}
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}

  {#if !compact}
    <p class="token-savings-privacy">{t("tokenSavings.privacy")}</p>
  {:else}
    <p class="token-savings-privacy">{t("tokenSavings.privacyShort")}</p>
  {/if}

  <button
    type="button"
    class="secondary token-savings-reset-btn"
    disabled={resetting || !hasData}
    onclick={handleReset}
  >
    {#key resetting}
      <span in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
        {resetting ? t("tokenSavings.resetting") : t("tokenSavings.reset")}
      </span>
    {/key}
  </button>
  {#if resetError}
    <p class="settings-hint deps-error" in:fade={hintFadeInParams} out:fade={hintFadeOutParams}>
      {resetError}
    </p>
  {/if}
</div>