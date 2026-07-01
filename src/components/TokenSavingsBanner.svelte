<script lang="ts">
  import { fade } from "svelte/transition";
  import { prefersReducedMotion } from "svelte/motion";
  import { hintFadeIn, hintFadeOut } from "../lib/motion";
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

  const reducedMotion = $derived(prefersReducedMotion.current);
  const fadeInParams = $derived(hintFadeIn(reducedMotion));
  const fadeOutParams = $derived(hintFadeOut(reducedMotion));

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
    in:fade={fadeInParams}
    out:fade={fadeOutParams}
    onclick={() => onOpenDetails?.()}
  >
    {label}
  </button>
{/if}