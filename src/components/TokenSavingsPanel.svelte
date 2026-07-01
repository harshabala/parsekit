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
  }: {
    stats: TokenStats | null;
    period: TokenStatsPeriod;
    onPeriodChange: (period: TokenStatsPeriod) => void;
    onStatsChange: (stats: TokenStats) => void;
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

<div class="token-savings-panel">
  <div class="settings-section-title">{t("tokenSavings.panelTitle")}</div>
  <p class="settings-hint">{t("tokenSavings.panelHint")}</p>

  <div class="token-savings-period" role="group" aria-label={t("tokenSavings.periodLabel")}>
    <div class="segmented-control token-savings-period-toggle">
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
    <p class="settings-hint token-savings-period-hint">{t("tokenSavings.periodHint")}</p>
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
      <div class="token-savings-stat token-savings-stat--pages">
        <dt>{t("tokenSavings.pagesUnlocked")}</dt>
        <dd>{formatTokenCount(stats?.total_pages_unlocked ?? 0)}</dd>
      </div>
      {#if (stats?.total_documents_unlocked ?? 0) > 0}
        <div class="token-savings-stat token-savings-stat--pages">
          <dt>{t("tokenSavings.documentsUnlocked")}</dt>
          <dd>{formatTokenCount(stats?.total_documents_unlocked ?? 0)}</dd>
        </div>
      {/if}
    </dl>

    {#if lifetimeTokens > 0}
      <p class="token-savings-approx">
        {t("tokenSavings.chatGptApprox", { count: formatTokenCount(chatGptApprox) })}
        <span class="token-savings-approx-label">{t("tokenSavings.approximateLabel")}</span>
      </p>
    {/if}

    {#if sortedFileTypes.length > 0}
      <div class="token-savings-by-type">
        <div class="token-savings-by-type-title">{t("tokenSavings.byTypeTitle")}</div>
        <ul class="token-savings-by-type-list">
          {#each sortedFileTypes as [type, row]}
            <li>
              {t("tokenSavings.byTypeRow", {
                type: fileTypeLabel(type),
                tokens: formatTokenCount(row.tokens_saved),
                files: formatTokenCount(row.files),
              })}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}

  <p class="token-savings-privacy">{t("tokenSavings.privacy")}</p>

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