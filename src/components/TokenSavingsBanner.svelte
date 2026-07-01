<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import {
    formatTokenCount,
    tokensForPeriod,
    type TokenStats,
  } from "../lib/tokenStats";
  import type { TokenStatsPeriod } from "../lib/store";

  let {
    stats,
    period,
    onOpenDetails,
  }: {
    stats: TokenStats | null;
    period: TokenStatsPeriod;
    onOpenDetails?: () => void;
  } = $props();

  const displayTokens = $derived(
    stats ? tokensForPeriod(stats, period) : 0,
  );
  const show = $derived(displayTokens > 0);
  const label = $derived(
    period === "month"
      ? t("tokenSavings.bannerMonth", {
          count: formatTokenCount(displayTokens),
        })
      : t("tokenSavings.bannerLifetime", {
          count: formatTokenCount(displayTokens),
        }),
  );
</script>

{#if show}
  <button
    type="button"
    class="token-savings-banner"
    aria-label={label}
    title={t("tokenSavings.bannerHint")}
    onclick={() => onOpenDetails?.()}
  >
    {label}
  </button>
{/if}